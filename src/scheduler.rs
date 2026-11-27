//! Cadence parser, next-run scheduler calculator, and atomic routine run history.
//!
//! Supports human-readable recurring schedules (`every 15m`, `every 30s`,
//! `hourly`, `daily 09:00`, `weekly mon`, or explicit `interval_secs`),
//! computes deterministic UTC `next_run_ms` timestamps with zero external
//! chrono dependencies, tracks pause/resume state (`enabled: bool`), and
//! persists `RoutineRunRecord` history atomically under `<person_dir>/routine_history.json`.

use crate::{backup_corrupt, records};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

pub const DEFAULT_SCHEDULER_TICK_SECS: u64 = 15;
pub const MIN_INTERVAL_SECS: u64 = 1;
pub const MAX_INTERVAL_SECS: u64 = 365 * 86_400;
pub const MAX_RUN_HISTORY: usize = 500;

static RUN_SEQ: AtomicU64 = AtomicU64::new(0);
static HISTORY_LOCK: Mutex<()> = Mutex::new(());

fn with_lock<T>(f: impl FnOnce() -> T) -> T {
    let _guard = HISTORY_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    f()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cadence {
    Manual,
    IntervalSecs(u64),
    Hourly { minute: u8 },
    Daily { hour: u8, minute: u8 },
    /// `weekday`: 0 = Mon, 1 = Tue, 2 = Wed, 3 = Thu, 4 = Fri, 5 = Sat, 6 = Sun.
    Weekly { weekday: u8, hour: u8, minute: u8 },
    Cron { expr: String, tz_offset_mins: i32 },
}

pub fn format_tz_offset(mins: i32) -> String {
    if mins == 0 {
        "UTC".to_string()
    } else {
        let sign = if mins >= 0 { '+' } else { '-' };
        let abs = mins.abs();
        format!("{sign}{:02}:{:02}", abs / 60, abs % 60)
    }
}

impl Cadence {
    pub fn canonical_label(&self) -> String {
        match self {
            Cadence::Manual => "manual".to_string(),
            Cadence::IntervalSecs(s) => {
                let s = *s;
                if s % 86_400 == 0 {
                    format!("every {}d", s / 86_400)
                } else if s % 3_600 == 0 {
                    format!("every {}h", s / 3_600)
                } else if s % 60 == 0 {
                    format!("every {}m", s / 60)
                } else {
                    format!("every {s}s")
                }
            }
            Cadence::Hourly { minute: 0 } => "hourly".to_string(),
            Cadence::Hourly { minute } => format!("hourly :{minute:02}"),
            Cadence::Daily { hour, minute } => format!("daily {hour:02}:{minute:02}"),
            Cadence::Weekly {
                weekday,
                hour,
                minute,
            } => {
                let day = weekday_short(*weekday);
                if *hour == 9 && *minute == 0 {
                    format!("weekly {day}")
                } else {
                    format!("weekly {day} {hour:02}:{minute:02}")
                }
            }
            Cadence::Cron {
                expr,
                tz_offset_mins,
            } => {
                if *tz_offset_mins == 0 {
                    expr.clone()
                } else {
                    format!("{expr} {}", format_tz_offset(*tz_offset_mins))
                }
            }
        }
    }

    pub fn interval_secs(&self) -> Option<u64> {
        match self {
            Cadence::Manual => None,
            Cadence::IntervalSecs(s) => Some(*s),
            Cadence::Hourly { .. } => Some(3_600),
            Cadence::Daily { .. } => Some(86_400),
            Cadence::Weekly { .. } => Some(7 * 86_400),
            Cadence::Cron { .. } => Some(60),
        }
    }

    /// Returns `Some(secs)` ONLY for relative `IntervalSecs` cadences (never for
    /// wall-clock `Hourly`/`Daily`/`Weekly`/`Cron` schedules, so persisted JSON never
    /// degrades wall-clock slots into relative intervals).
    pub fn explicit_interval_secs(&self) -> Option<u64> {
        match self {
            Cadence::IntervalSecs(s) => Some(*s),
            _ => None,
        }
    }

    pub fn is_wall_clock(&self) -> bool {
        matches!(
            self,
            Cadence::Hourly { .. }
                | Cadence::Daily { .. }
                | Cadence::Weekly { .. }
                | Cadence::Cron { .. }
        )
    }

    pub fn is_scheduled(&self) -> bool {
        !matches!(self, Cadence::Manual)
    }
}

fn weekday_short(w: u8) -> &'static str {
    match w {
        0 => "mon",
        1 => "tue",
        2 => "wed",
        3 => "thu",
        4 => "fri",
        5 => "sat",
        _ => "sun",
    }
}

fn parse_weekday(tok: &str) -> Option<u8> {
    match tok.trim().to_ascii_lowercase().as_str() {
        "mon" | "monday" => Some(0),
        "tue" | "tues" | "tuesday" => Some(1),
        "wed" | "weds" | "wednesday" => Some(2),
        "thu" | "thur" | "thurs" | "thursday" => Some(3),
        "fri" | "friday" => Some(4),
        "sat" | "saturday" => Some(5),
        "sun" | "sunday" => Some(6),
        _ => None,
    }
}

fn parse_hhmm(tok: &str) -> Result<(u8, u8), String> {
    let Some((h_str, m_str)) = tok.trim().split_once(':') else {
        return Err(format!("invalid time '{tok}', expected HH:MM"));
    };
    let hour: u8 = h_str
        .parse()
        .map_err(|_| format!("invalid hour in '{tok}'"))?;
    let minute: u8 = m_str
        .parse()
        .map_err(|_| format!("invalid minute in '{tok}'"))?;
    if hour > 23 || minute > 59 {
        return Err(format!("time out of range in '{tok}' (expected 00:00..23:59)"));
    }
    Ok((hour, minute))
}

fn parse_duration_secs(tokens: &[&str]) -> Result<u64, String> {
    if tokens.is_empty() {
        return Err("missing duration after 'every'".to_string());
    }
    let (num_str, unit_str) = if tokens.len() == 1 {
        let s = tokens[0];
        let split_at = s
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(s.len());
        (&s[..split_at], &s[split_at..])
    } else if tokens.len() == 2 {
        (tokens[0], tokens[1])
    } else {
        return Err(format!("unexpected extra tokens in duration: {}", tokens.join(" ")));
    };

    let amount: u64 = num_str
        .parse()
        .map_err(|_| format!("invalid interval number '{num_str}'"))?;
    if amount == 0 {
        return Err("interval must be greater than zero".to_string());
    }
    let mult: u64 = match unit_str.trim().to_ascii_lowercase().as_str() {
        "" | "s" | "sec" | "secs" | "second" | "seconds" => 1,
        "m" | "min" | "mins" | "minute" | "minutes" => 60,
        "h" | "hr" | "hrs" | "hour" | "hours" => 3_600,
        "d" | "day" | "days" => 86_400,
        other => {
            return Err(format!(
                "unsupported interval unit '{other}' (use s, m, h, or d)"
            ))
        }
    };
    let secs = amount
        .checked_mul(mult)
        .ok_or_else(|| "interval overflows maximum duration".to_string())?;
    if !(MIN_INTERVAL_SECS..=MAX_INTERVAL_SECS).contains(&secs) {
        return Err(format!(
            "interval must be between {MIN_INTERVAL_SECS}s and {MAX_INTERVAL_SECS}s"
        ));
    }
    Ok(secs)
}

pub fn parse_tz_offset_mins(raw: &str) -> Result<i32, String> {
    let mut s = raw.trim();
    if let Some(rest) = s
        .strip_prefix("tz:")
        .or_else(|| s.strip_prefix("TZ:"))
        .or_else(|| s.strip_prefix("tz="))
        .or_else(|| s.strip_prefix("TZ="))
    {
        s = rest.trim();
    }
    if s.is_empty() {
        return Err("timezone offset cannot be empty".to_string());
    }
    let low = s.to_ascii_lowercase();
    let named = match low.as_str() {
        "z" | "utc" | "gmt" | "etc/utc" | "etc/gmt" => Some(0),
        "est" | "america/new_york" | "america/toronto" => Some(-300),
        "edt" => Some(-240),
        "cst" | "america/chicago" => Some(-360),
        "cdt" => Some(-300),
        "mst" | "america/denver" | "america/phoenix" => Some(-420),
        "mdt" => Some(-360),
        "pst" | "america/los_angeles" => Some(-480),
        "pdt" => Some(-420),
        "europe/london" => Some(0),
        "cet" | "europe/berlin" | "europe/paris" => Some(60),
        "cest" => Some(120),
        "ist" | "asia/kolkata" => Some(330),
        "sgt" | "asia/singapore" => Some(480),
        "jst" | "asia/tokyo" => Some(540),
        "aest" | "australia/sydney" => Some(600),
        "nzst" | "pacific/auckland" => Some(720),
        _ => None,
    };
    if let Some(m) = named {
        return Ok(m);
    }

    let trimmed = if let Some(rest) = low.strip_prefix("utc").or_else(|| low.strip_prefix("gmt")) {
        rest.trim()
    } else {
        s
    };
    if trimmed == "0" || trimmed == "+0" || trimmed == "-0" {
        return Ok(0);
    }
    let (sign, rest) = if let Some(r) = trimmed.strip_prefix('+') {
        (1i32, r)
    } else if let Some(r) = trimmed.strip_prefix('-') {
        (-1i32, r)
    } else {
        return Err(format!("unsupported timezone '{raw}'"));
    };
    let (h, m) = if let Some((hs, ms)) = rest.split_once(':') {
        let h: i32 = hs
            .parse()
            .map_err(|_| format!("invalid timezone hour in '{raw}'"))?;
        let m: i32 = ms
            .parse()
            .map_err(|_| format!("invalid timezone minute in '{raw}'"))?;
        (h, m)
    } else if rest.len() == 4 && rest.bytes().all(|b| b.is_ascii_digit()) {
        let h: i32 = rest[..2]
            .parse()
            .map_err(|_| format!("invalid timezone hour in '{raw}'"))?;
        let m: i32 = rest[2..]
            .parse()
            .map_err(|_| format!("invalid timezone minute in '{raw}'"))?;
        (h, m)
    } else if !rest.is_empty() && rest.len() <= 2 && rest.bytes().all(|b| b.is_ascii_digit()) {
        let h: i32 = rest
            .parse()
            .map_err(|_| format!("invalid timezone hour in '{raw}'"))?;
        (h, 0)
    } else {
        return Err(format!("invalid timezone offset '{raw}'"));
    };
    if !(0..=14).contains(&h) || !(0..=59).contains(&m) {
        return Err(format!("timezone offset '{raw}' out of range (-12:00..+14:00)"));
    }
    let total = sign * (h * 60 + m);
    if !(-720..=840).contains(&total) {
        return Err(format!("timezone offset '{raw}' out of range (-12:00..+14:00)"));
    }
    Ok(total)
}

#[derive(Clone, Debug)]
struct CronSpec {
    minutes: u64,   // bits 0..=59
    hours: u32,     // bits 0..=23
    doms: u32,      // bits 1..=31
    months: u16,    // bits 1..=12
    dows: u8,       // bits 0..=6 (0 = Sun, 1 = Mon .. 6 = Sat)
    dom_restricted: bool,
    dow_restricted: bool,
}

fn parse_cron_atom(tok: &str, min: u8, max: u8, is_month: bool, is_dow: bool) -> Result<u8, String> {
    let low = tok.trim().to_ascii_lowercase();
    if is_month {
        match low.as_str() {
            "jan" => return Ok(1),
            "feb" => return Ok(2),
            "mar" => return Ok(3),
            "apr" => return Ok(4),
            "may" => return Ok(5),
            "jun" => return Ok(6),
            "jul" => return Ok(7),
            "aug" => return Ok(8),
            "sep" => return Ok(9),
            "oct" => return Ok(10),
            "nov" => return Ok(11),
            "dec" => return Ok(12),
            _ => {}
        }
    }
    if is_dow {
        match low.as_str() {
            "sun" => return Ok(0),
            "mon" => return Ok(1),
            "tue" => return Ok(2),
            "wed" => return Ok(3),
            "thu" => return Ok(4),
            "fri" => return Ok(5),
            "sat" => return Ok(6),
            _ => {}
        }
    }
    let val: u8 = low
        .parse()
        .map_err(|_| format!("invalid cron token '{tok}'"))?;
    let norm = if is_dow && val == 7 { 0 } else { val };
    if norm < min || norm > max {
        return Err(format!(
            "cron value '{tok}' out of range ({min}..={max})"
        ));
    }
    Ok(norm)
}

fn parse_cron_field(
    field: &str,
    min: u8,
    max: u8,
    is_month: bool,
    is_dow: bool,
) -> Result<(u64, bool), String> {
    let trimmed = field.trim();
    if trimmed.is_empty() {
        return Err("empty cron field".to_string());
    }
    let restricted = trimmed != "*";
    let mut mask: u64 = 0;
    for part in trimmed.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return Err(format!("empty list element in cron field '{field}'"));
        }
        let (range_part, step) = if let Some((l, r)) = part.split_once('/') {
            let st: u8 = r
                .trim()
                .parse()
                .map_err(|_| format!("invalid cron step in '{part}'"))?;
            if st == 0 {
                return Err(format!("cron step must be > 0 in '{part}'"));
            }
            (l.trim(), st)
        } else {
            (part, 1u8)
        };
        if range_part == "*" {
            let mut v = min;
            while v <= max {
                mask |= 1u64 << v;
                v = match v.checked_add(step) {
                    Some(n) => n,
                    None => break,
                };
            }
        } else if let Some((a_s, b_s)) = range_part.split_once('-') {
            let a = parse_cron_atom(a_s, min, max, is_month, is_dow)?;
            let mut b = parse_cron_atom(b_s, min, max, is_month, is_dow)?;
            if is_dow && b == 0 && a > 0 && b_s.trim() == "7" {
                // Allow 1-7 or 5-7 where 7 means Sunday.
                b = 7;
            }
            if a > b {
                return Err(format!("cron range start > end in '{part}'"));
            }
            let mut v = a;
            while v <= b {
                let bit = if is_dow && v == 7 { 0 } else { v };
                mask |= 1u64 << bit;
                v = match v.checked_add(step) {
                    Some(n) => n,
                    None => break,
                };
            }
        } else {
            let a = parse_cron_atom(range_part, min, max, is_month, is_dow)?;
            if part.contains('/') {
                let mut v = a;
                while v <= max {
                    mask |= 1u64 << v;
                    v = match v.checked_add(step) {
                        Some(n) => n,
                        None => break,
                    };
                }
            } else {
                mask |= 1u64 << a;
            }
        }
    }
    if mask == 0 {
        return Err(format!("cron field '{field}' matched no values"));
    }
    Ok((mask, restricted))
}

impl CronSpec {
    fn parse(expr: &str) -> Result<Self, String> {
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(format!(
                "cron expression must have 5 fields (min hour dom month dow), got '{expr}'"
            ));
        }
        let (min_mask, _) = parse_cron_field(parts[0], 0, 59, false, false)?;
        let (hour_mask, _) = parse_cron_field(parts[1], 0, 23, false, false)?;
        let (dom_mask, dom_restricted) = parse_cron_field(parts[2], 1, 31, false, false)?;
        let (month_mask, _) = parse_cron_field(parts[3], 1, 12, true, false)?;
        let (dow_mask, dow_restricted) = parse_cron_field(parts[4], 0, 6, false, true)?;

        let spec = Self {
            minutes: min_mask,
            hours: hour_mask as u32,
            doms: dom_mask as u32,
            months: month_mask as u16,
            dows: dow_mask as u8,
            dom_restricted,
            dow_restricted,
        };

        // Reject impossible fixed month/day combinations like Feb 30 or Apr 31 when dow isn't open.
        if dom_restricted && !dow_restricted {
            let max_days = [0u8, 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            let mut any_valid = false;
            for m in 1u8..=12 {
                if (spec.months & (1u16 << m)) == 0 {
                    continue;
                }
                for d in 1u8..=max_days[m as usize] {
                    if (spec.doms & (1u32 << d)) != 0 {
                        any_valid = true;
                        break;
                    }
                }
                if any_valid {
                    break;
                }
            }
            if !any_valid {
                return Err(format!("cron expression '{expr}' matches no calendar date"));
            }
        }
        Ok(spec)
    }

    fn day_matches(&self, dom: u8, cron_dow: u8) -> bool {
        let dom_ok = (self.doms & (1u32 << dom)) != 0;
        let dow_ok = (self.dows & (1u8 << cron_dow)) != 0;
        if self.dom_restricted && self.dow_restricted {
            dom_ok || dow_ok
        } else {
            dom_ok && dow_ok
        }
    }
}

/// Parse a schedule string (`manual`, `every 15m`, `every 30s`, `hourly`,
/// `daily 09:00`, `weekly mon`, `weekly fri 17:30`, raw seconds, or 5-field
/// cron `min hour dom month dow` with optional timezone offset).
pub fn parse_cadence(spec: &str) -> Result<Cadence, String> {
    let trimmed = spec.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.is_empty() || matches!(lower.as_str(), "manual" | "off" | "none" | "on-demand") {
        return Ok(Cadence::Manual);
    }

    let tokens_orig: Vec<&str> = trimmed.split_whitespace().collect();
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    match tokens[0] {
        "every" => {
            let secs = parse_duration_secs(&tokens[1..])?;
            Ok(Cadence::IntervalSecs(secs))
        }
        "hourly" => {
            if tokens.len() == 1 {
                return Ok(Cadence::Hourly { minute: 0 });
            }
            if tokens.len() == 2 {
                let raw_m = tokens[1].trim_start_matches(':');
                let minute: u8 = raw_m
                    .parse()
                    .map_err(|_| format!("invalid hourly minute '{}'", tokens[1]))?;
                if minute > 59 {
                    return Err(format!("minute out of range: {minute}"));
                }
                return Ok(Cadence::Hourly { minute });
            }
            Err(format!("invalid hourly cadence: '{spec}'"))
        }
        "daily" => {
            if tokens.len() == 1 {
                return Ok(Cadence::Daily { hour: 9, minute: 0 });
            }
            if tokens.len() == 2 {
                let (hour, minute) = parse_hhmm(tokens[1])?;
                return Ok(Cadence::Daily { hour, minute });
            }
            Err(format!("invalid daily cadence: '{spec}'"))
        }
        "weekly" => {
            if tokens.len() == 1 {
                return Ok(Cadence::Weekly {
                    weekday: 0,
                    hour: 9,
                    minute: 0,
                });
            }
            let weekday = parse_weekday(tokens[1])
                .ok_or_else(|| format!("invalid weekday '{}' in '{spec}'", tokens[1]))?;
            if tokens.len() == 2 {
                return Ok(Cadence::Weekly {
                    weekday,
                    hour: 9,
                    minute: 0,
                });
            }
            if tokens.len() == 3 {
                let (hour, minute) = parse_hhmm(tokens[2])?;
                return Ok(Cadence::Weekly {
                    weekday,
                    hour,
                    minute,
                });
            }
            Err(format!("invalid weekly cadence: '{spec}'"))
        }
        _ => {
            if tokens_orig.len() == 5 {
                let expr = tokens_orig.join(" ");
                CronSpec::parse(&expr)?;
                return Ok(Cadence::Cron {
                    expr,
                    tz_offset_mins: 0,
                });
            }
            if tokens_orig.len() == 6 {
                let expr = tokens_orig[..5].join(" ");
                if let Ok(tz_offset_mins) = parse_tz_offset_mins(tokens_orig[5]) {
                    CronSpec::parse(&expr)?;
                    return Ok(Cadence::Cron {
                        expr,
                        tz_offset_mins,
                    });
                }
            }
            // Support compact forms like "15m", "300s", "900".
            if tokens.len() == 1 {
                let secs = parse_duration_secs(&tokens)?;
                return Ok(Cadence::IntervalSecs(secs));
            }
            Err(format!(
                "unsupported cadence '{spec}' (use 'manual', 'every 15m', 'hourly', 'daily 09:00', 'weekly mon', or 5-field cron)"
            ))
        }
    }
}

pub fn parse_cadence_with_tz(spec: &str, tz_offset_mins: Option<i32>) -> Result<Cadence, String> {
    let mut cadence = parse_cadence(spec)?;
    if let Some(tz) = tz_offset_mins {
        if !(-720..=840).contains(&tz) {
            return Err("tz_offset_mins must be between -720 and 840".to_string());
        }
        if let Cadence::Cron {
            tz_offset_mins: ref mut cur,
            ..
        } = cadence
        {
            *cur = tz;
        }
    }
    Ok(cadence)
}

/// Resolve cadence from an optional `schedule` string and/or explicit
/// `interval_secs` integer (explicit `interval_secs` takes precedence when > 0).
pub fn parse_cadence_or_interval(
    schedule: Option<&str>,
    interval_secs: Option<u64>,
) -> Result<Cadence, String> {
    parse_cadence_or_interval_with_tz(schedule, interval_secs, None)
}

pub fn parse_cadence_or_interval_with_tz(
    schedule: Option<&str>,
    interval_secs: Option<u64>,
    tz_offset_mins: Option<i32>,
) -> Result<Cadence, String> {
    if let Some(secs) = interval_secs {
        if !(MIN_INTERVAL_SECS..=MAX_INTERVAL_SECS).contains(&secs) {
            return Err(format!(
                "interval_secs must be between {MIN_INTERVAL_SECS} and {MAX_INTERVAL_SECS}"
            ));
        }
        return Ok(Cadence::IntervalSecs(secs));
    }
    parse_cadence_with_tz(schedule.unwrap_or("manual"), tz_offset_mins)
}

/// Howard Hinnant's exact civil date -> days since 1970-01-01 (UTC).
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let m = month as u64;
    let d = day as u64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + (doe as i64) - 719_468
}

/// Howard Hinnant's exact days since 1970-01-01 -> (year, month, day).
fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

/// Day of week from days since Unix epoch: 0 = Monday .. 6 = Sunday.
fn weekday_from_days(days_since_epoch: i64) -> u8 {
    (days_since_epoch + 3).rem_euclid(7) as u8
}

fn utc_day_ms(days: i64, hour: u8, minute: u8) -> u64 {
    let secs = days
        .saturating_mul(86_400)
        .saturating_add((hour as i64) * 3_600)
        .saturating_add((minute as i64) * 60);
    if secs <= 0 {
        0
    } else {
        (secs as u64).saturating_mul(1_000)
    }
}

/// Compute the next scheduled execution timestamp (Unix millis UTC) strictly
/// after `anchor = last_run_ms.unwrap_or(now_ms)`. Returns `None` for `Manual`.
pub fn next_run_ms(cadence: &Cadence, last_run_ms: Option<u64>, now_ms: u64) -> Option<u64> {
    let anchor = last_run_ms.filter(|&t| t > 0).unwrap_or(now_ms);
    match cadence {
        Cadence::Manual => None,
        Cadence::IntervalSecs(secs) => Some(anchor.saturating_add(secs.saturating_mul(1_000))),
        Cadence::Hourly { minute } => {
            let hour_ms: u64 = 3_600_000;
            let hour_floor = (anchor / hour_ms) * hour_ms;
            let candidate = hour_floor.saturating_add((*minute as u64) * 60_000);
            if candidate > anchor {
                Some(candidate)
            } else {
                Some(candidate.saturating_add(hour_ms))
            }
        }
        Cadence::Daily { hour, minute } => {
            let (y, m, d) = records::civil_from_millis(anchor);
            let day = days_from_civil(y, m, d);
            let today_slot = utc_day_ms(day, *hour, *minute);
            if today_slot > anchor {
                Some(today_slot)
            } else {
                Some(utc_day_ms(day + 1, *hour, *minute))
            }
        }
        Cadence::Weekly {
            weekday,
            hour,
            minute,
        } => {
            let (y, m, d) = records::civil_from_millis(anchor);
            let day = days_from_civil(y, m, d);
            let cur_w = weekday_from_days(day);
            let delta = ((*weekday as i64) - (cur_w as i64)).rem_euclid(7);
            let candidate = utc_day_ms(day + delta, *hour, *minute);
            if candidate > anchor {
                Some(candidate)
            } else {
                Some(utc_day_ms(day + delta + 7, *hour, *minute))
            }
        }
        Cadence::Cron {
            expr,
            tz_offset_mins,
        } => {
            let spec = CronSpec::parse(expr).ok()?;
            let offset_ms = (*tz_offset_mins as i64) * 60_000;
            let local_anchor_min = (anchor as i64)
                .saturating_add(offset_ms)
                .div_euclid(60_000)
                .saturating_add(1);
            let start_day = local_anchor_min.div_euclid(1_440);
            let start_min_of_day = local_anchor_min.rem_euclid(1_440);

            for day_offset in 0..=14_640i64 {
                let day = start_day.saturating_add(day_offset);
                let (_y, m, d) = civil_from_days(day);
                if (spec.months & (1u16 << (m as u8))) == 0 {
                    continue;
                }
                let cron_dow = (weekday_from_days(day) + 1) % 7;
                if !spec.day_matches(d as u8, cron_dow) {
                    continue;
                }
                let first_min = if day_offset == 0 {
                    start_min_of_day
                } else {
                    0
                };
                let h_start = (first_min / 60) as u8;
                for h in h_start..24 {
                    if (spec.hours & (1u32 << h)) == 0 {
                        continue;
                    }
                    let m_start = if day_offset == 0 && h == h_start {
                        (first_min % 60) as u8
                    } else {
                        0
                    };
                    for min in m_start..60 {
                        if (spec.minutes & (1u64 << min)) != 0 {
                            let local_secs = day
                                .checked_mul(86_400)
                                .and_then(|s| s.checked_add((h as i64) * 3_600))
                                .and_then(|s| s.checked_add((min as i64) * 60));
                            let local_ms = local_secs.and_then(|s| s.checked_mul(1_000))?;
                            let utc_ms = local_ms.saturating_sub(offset_ms);
                            if utc_ms >= 0 && (utc_ms as u64) > anchor {
                                return Some(utc_ms as u64);
                            }
                        }
                    }
                }
            }
            None
        }
    }
}

fn routine_tz_offset_mins(routine: &Value) -> Option<i32> {
    if let Some(v) = routine.get("tz_offset_mins").and_then(|x| x.as_i64()) {
        if (-720..=840).contains(&v) {
            return Some(v as i32);
        }
    }
    for key in ["tz", "timezone"] {
        if let Some(s) = routine.get(key).and_then(|x| x.as_str()) {
            if let Ok(m) = parse_tz_offset_mins(s) {
                return Some(m);
            }
        }
    }
    None
}

/// Resolve a persisted routine's cadence. Prefers a wall-clock `schedule`
/// (`hourly`, `daily 09:00`, `weekly mon`, 5-field cron) even if a legacy record
/// also carries a stale `interval_secs` field, otherwise resolves via
/// `parse_cadence_or_interval_with_tz`.
pub fn parse_routine_cadence(routine: &Value) -> Result<Cadence, String> {
    let tz_opt = routine_tz_offset_mins(routine);
    if let Some(sched) = routine["schedule"].as_str() {
        if let Ok(c) = parse_cadence_with_tz(sched, tz_opt) {
            if c.is_wall_clock() {
                return Ok(c);
            }
        }
    }
    parse_cadence_or_interval_with_tz(
        routine["schedule"].as_str(),
        routine["interval_secs"].as_u64(),
        tz_opt,
    )
}

/// Compute `next_run_ms` for a persisted routine record (accounting for
/// `enabled`, `created_ms`, and `last_run_ms`). Returns `None` when paused or
/// manual.
pub fn compute_routine_next_run_ms(routine: &Value, now_ms: u64) -> Option<u64> {
    if !routine["enabled"].as_bool().unwrap_or(true) {
        return None;
    }
    let cadence = parse_routine_cadence(routine).ok()?;
    if !cadence.is_scheduled() {
        return None;
    }
    let last_run = routine["last_run_ms"].as_u64().filter(|&t| t > 0);
    let created = routine["created_ms"].as_u64().filter(|&t| t > 0);
    let anchor = last_run.or(created).unwrap_or(now_ms);
    next_run_ms(&cadence, Some(anchor), now_ms)
}

/// Whether `routine` is enabled, has a valid recurring cadence, and is due at `now_ms`.
pub fn is_routine_due(routine: &Value, now_ms: u64) -> bool {
    compute_routine_next_run_ms(routine, now_ms)
        .map(|due_at| due_at <= now_ms)
        .unwrap_or(false)
}

/// Filter and sort all due routines (earliest `next_run_ms` first).
pub fn find_due_routines(routines: &[Value], now_ms: u64) -> Vec<Value> {
    let mut due: Vec<(u64, Value)> = routines
        .iter()
        .filter_map(|r| {
            let next = compute_routine_next_run_ms(r, now_ms)?;
            if next <= now_ms {
                Some((next, r.clone()))
            } else {
                None
            }
        })
        .collect();
    due.sort_by_key(|(next, _)| *next);
    due.into_iter().map(|(_, r)| r).collect()
}

/// Whether background ticking is disabled via `ERGON_DISABLE_SCHEDULER=1|true|yes`.
pub fn is_scheduler_disabled_by_env() -> bool {
    std::env::var("ERGON_DISABLE_SCHEDULER")
        .map(|v| {
            let s = v.trim();
            s == "1" || s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes")
        })
        .unwrap_or(false)
}

/// Helper to resolve optional timezone offset from patch or existing routine object.
pub fn resolve_tz_from_patch(
    patch: &Value,
    existing: &serde_json::Map<String, Value>,
) -> Result<Option<i32>, String> {
    if let Some(v) = patch.get("tz_offset_mins") {
        if v.is_null() {
            return Ok(None);
        }
        let Some(n) = v.as_i64() else {
            return Err("tz_offset_mins must be an integer".to_string());
        };
        if !(-720..=840).contains(&n) {
            return Err("tz_offset_mins must be between -720 and 840".to_string());
        }
        return Ok(Some(n as i32));
    }
    for key in ["tz", "timezone"] {
        if let Some(v) = patch.get(key) {
            if v.is_null() {
                return Ok(None);
            }
            let Some(s) = v.as_str() else {
                return Err(format!("{key} must be a string"));
            };
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            return Ok(Some(parse_tz_offset_mins(trimmed)?));
        }
    }
    let embedded_tz = patch
        .get("schedule")
        .and_then(|v| v.as_str())
        .is_some_and(|s| s.split_whitespace().count() == 6);
    if !embedded_tz {
        if let Some(n) = existing.get("tz_offset_mins").and_then(|v| v.as_i64()) {
            if (-720..=840).contains(&n) {
                return Ok(Some(n as i32));
            }
        }
    }
    Ok(None)
}

/// Apply a `PATCH /api/routines/:id` JSON payload to a routine object in-place.
/// Supports `enabled`, `schedule`, `interval_secs`, `name`, `prompt`, `skill_id`.
pub fn apply_routine_patch(routine: &mut Value, patch: &Value) -> Result<(), String> {
    let Some(obj) = routine.as_object_mut() else {
        return Err("routine record must be a JSON object".to_string());
    };
    if let Some(enabled) = patch.get("enabled").and_then(|v| v.as_bool()) {
        obj.insert("enabled".to_string(), json!(enabled));
    }
    if let Some(name) = patch.get("name").and_then(|v| v.as_str()) {
        let clean: String = name
            .replace(['\r', '\n'], " ")
            .trim()
            .chars()
            .take(120)
            .collect();
        if clean.is_empty() {
            return Err("routine name cannot be empty".to_string());
        }
        obj.insert("name".to_string(), json!(clean));
    }
    if let Some(prompt) = patch.get("prompt").and_then(|v| v.as_str()) {
        let clean: String = prompt.trim().chars().take(4000).collect();
        if clean.is_empty() {
            return Err("routine prompt cannot be empty".to_string());
        }
        obj.insert("prompt".to_string(), json!(clean));
    }
    if patch.get("schedule").is_some()
        || patch.get("interval_secs").is_some()
        || patch.get("tz_offset_mins").is_some()
        || patch.get("tz").is_some()
        || patch.get("timezone").is_some()
    {
        let interval = patch.get("interval_secs").and_then(|v| v.as_u64());
        let sched = patch
            .get("schedule")
            .and_then(|v| v.as_str())
            .or_else(|| obj.get("schedule").and_then(|v| v.as_str()));
        if patch.get("interval_secs").is_some()
            && interval.is_none()
            && !patch["interval_secs"].is_null()
        {
            return Err("interval_secs must be a positive integer".to_string());
        }
        let tz_opt = resolve_tz_from_patch(patch, obj)?;
        let cadence = parse_cadence_or_interval_with_tz(sched, interval, tz_opt)?;
        match &cadence {
            Cadence::Cron {
                expr,
                tz_offset_mins,
            } => {
                obj.insert("schedule".to_string(), json!(expr));
                obj.insert("tz_offset_mins".to_string(), json!(tz_offset_mins));
                let tz_str = patch
                    .get("tz")
                    .or_else(|| patch.get("timezone"))
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .unwrap_or_else(|| format_tz_offset(*tz_offset_mins));
                obj.insert("tz".to_string(), json!(tz_str));
                obj.remove("interval_secs");
            }
            _ => {
                obj.insert("schedule".to_string(), json!(cadence.canonical_label()));
                obj.remove("tz_offset_mins");
                obj.remove("tz");
                match cadence.explicit_interval_secs() {
                    Some(secs) => {
                        obj.insert("interval_secs".to_string(), json!(secs));
                    }
                    None => {
                        obj.remove("interval_secs");
                    }
                }
            }
        }
    }
    if let Some(skill) = patch.get("skill_id") {
        if skill.is_null() {
            obj.insert("skill_id".to_string(), Value::Null);
        } else if let Some(s) = skill.as_str() {
            let clean: String = s.trim().chars().take(64).collect();
            obj.insert(
                "skill_id".to_string(),
                if clean.is_empty() {
                    Value::Null
                } else {
                    json!(clean)
                },
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Atomic Routine Run History Store (`routine_history.json`)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutineRunRecord {
    pub run_id: String,
    pub routine_id: String,
    pub routine_name: String,
    pub started_ms: u64,
    pub finished_ms: u64,
    pub duration_ms: u64,
    pub status: String, // "ok" | "error" | "cancelled"
    pub receipt_id: Option<String>,
    pub thread_id: Option<String>,
    pub summary: String,
    pub trigger: String, // "scheduled" | "manual"
    pub error: Option<String>,
}

impl RoutineRunRecord {
    pub fn to_json(&self) -> Value {
        json!({
            "run_id": self.run_id,
            "routine_id": self.routine_id,
            "routine_name": self.routine_name,
            "started_ms": self.started_ms,
            "finished_ms": self.finished_ms,
            "duration_ms": self.duration_ms,
            "status": self.status,
            "receipt_id": self.receipt_id,
            "thread_id": self.thread_id,
            "summary": self.summary,
            "trigger": self.trigger,
            "error": self.error,
        })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let started_ms = v["started_ms"].as_u64().unwrap_or(0);
        let finished_ms = v["finished_ms"].as_u64().unwrap_or(started_ms);
        let duration_ms = v["duration_ms"]
            .as_u64()
            .unwrap_or_else(|| finished_ms.saturating_sub(started_ms));
        Some(RoutineRunRecord {
            run_id: v["run_id"].as_str()?.to_string(),
            routine_id: v["routine_id"].as_str()?.to_string(),
            routine_name: v["routine_name"].as_str().unwrap_or("").to_string(),
            started_ms,
            finished_ms,
            duration_ms,
            status: v["status"].as_str().unwrap_or("ok").to_string(),
            receipt_id: v["receipt_id"].as_str().map(|s| s.to_string()),
            thread_id: v["thread_id"].as_str().map(|s| s.to_string()),
            summary: v["summary"].as_str().unwrap_or("").to_string(),
            trigger: v["trigger"].as_str().unwrap_or("scheduled").to_string(),
            error: v["error"].as_str().map(|s| s.to_string()),
        })
    }
}

#[allow(clippy::too_many_arguments)]
pub fn new_run_record(
    routine_id: &str,
    routine_name: &str,
    started_ms: u64,
    finished_ms: u64,
    status: &str,
    receipt_id: Option<String>,
    thread_id: Option<String>,
    summary: &str,
    trigger: &str,
    error: Option<String>,
) -> RoutineRunRecord {
    let seq = RUN_SEQ.fetch_add(1, Ordering::SeqCst);
    let end = finished_ms.max(started_ms);
    RoutineRunRecord {
        run_id: format!("rrun-{started_ms}-{seq}"),
        routine_id: routine_id.to_string(),
        routine_name: routine_name.to_string(),
        started_ms,
        finished_ms: end,
        duration_ms: end.saturating_sub(started_ms),
        status: status.to_string(),
        receipt_id,
        thread_id,
        summary: summary.chars().take(1000).collect(),
        trigger: trigger.to_string(),
        error: error.map(|e| e.chars().take(500).collect()),
    }
}

pub fn history_path(person_data_dir: &Path) -> PathBuf {
    person_data_dir.join("routine_history.json")
}

fn load_history_unlocked(path: &Path) -> Vec<RoutineRunRecord> {
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        let _ = backup_corrupt(path);
        return Vec::new();
    };
    let parsed = serde_json::from_str::<Value>(text);
    let arr = parsed
        .as_ref()
        .ok()
        .and_then(|v| v["runs"].as_array());
    match arr {
        Some(items) => items
            .iter()
            .filter_map(RoutineRunRecord::from_json)
            .collect(),
        None => {
            let _ = backup_corrupt(path);
            Vec::new()
        }
    }
}

fn save_history_unlocked(path: &Path, runs: &[RoutineRunRecord]) -> Result<(), String> {
    let body = json!({
        "runs": runs.iter().map(|r| r.to_json()).collect::<Vec<_>>(),
    })
    .to_string();
    crate::atomic_write_bytes(path, body.as_bytes())
        .map_err(|e| format!("write routine history: {e}"))
}

/// Append one `RoutineRunRecord` atomically (capping stored history at `MAX_RUN_HISTORY`).
pub fn record_run_history(
    person_data_dir: &Path,
    record: RoutineRunRecord,
) -> Result<RoutineRunRecord, String> {
    with_lock(|| {
        let path = history_path(person_data_dir);
        let mut runs = load_history_unlocked(&path);
        runs.push(record.clone());
        if runs.len() > MAX_RUN_HISTORY {
            let excess = runs.len() - MAX_RUN_HISTORY;
            runs.drain(0..excess);
        }
        save_history_unlocked(&path, &runs)?;
        Ok(record)
    })
}

/// List routine run history newest-first, optionally filtered to one `routine_id`.
pub fn list_run_history(
    person_data_dir: &Path,
    routine_id: Option<&str>,
    limit: usize,
) -> Vec<RoutineRunRecord> {
    with_lock(|| {
        let mut runs = load_history_unlocked(&history_path(person_data_dir));
        runs.reverse();
        if let Some(rid) = routine_id {
            runs.retain(|r| r.routine_id == rid);
        }
        if limit > 0 && runs.len() > limit {
            runs.truncate(limit);
        }
        runs
    })
}

/// Count how many scheduled cadence windows have been missed up to `now_ms` (capped at 99).
pub fn count_missed_windows(routine: &Value, now_ms: u64) -> usize {
    if !routine["enabled"].as_bool().unwrap_or(true) {
        return 0;
    }
    let Ok(cadence) = parse_routine_cadence(routine) else {
        return 0;
    };
    if !cadence.is_scheduled() {
        return 0;
    }
    let Some(mut cur) = routine["last_run_ms"]
        .as_u64()
        .filter(|&t| t > 0)
        .or_else(|| routine["created_ms"].as_u64().filter(|&t| t > 0))
    else {
        return 0;
    };
    let mut missed = 0usize;
    while missed < 99 {
        let Some(slot) = next_run_ms(&cadence, Some(cur), now_ms) else {
            break;
        };
        if slot > now_ms || slot <= cur {
            break;
        }
        missed += 1;
        cur = slot;
    }
    missed
}

/// Enrich a routine JSON record with live scheduler fields (`next_run_ms`,
/// `cadence_label`, `interval_secs`, `due_now`, `missed_windows`, `in_seconds`, `history_count`).
pub fn enrich_routine_json(routine: &Value, now_ms: u64, history: &[RoutineRunRecord]) -> Value {
    let mut out = routine.clone();
    let enabled = routine["enabled"].as_bool().unwrap_or(true);
    let parsed = parse_routine_cadence(routine);
    let (cadence_valid, cadence_label, interval_secs, cron_tz) = match &parsed {
        Ok(c) => (
            true,
            c.canonical_label(),
            c.explicit_interval_secs(),
            match c {
                Cadence::Cron { tz_offset_mins, .. } => Some(*tz_offset_mins),
                _ => None,
            },
        ),
        Err(_) => (
            false,
            routine["schedule"].as_str().unwrap_or("manual").to_string(),
            None,
            None,
        ),
    };
    let next = compute_routine_next_run_ms(routine, now_ms);
    let due_now = enabled && next.map(|n| n <= now_ms).unwrap_or(false);
    let missed_windows = count_missed_windows(routine, now_ms);
    let in_seconds = next.map(|n| n.saturating_sub(now_ms) / 1_000);

    let rid = routine["id"].as_str().unwrap_or("");
    let matching: Vec<&RoutineRunRecord> = history.iter().filter(|h| h.routine_id == rid).collect();
    let last_receipt_id = matching.first().and_then(|h| h.receipt_id.clone());

    if let Some(obj) = out.as_object_mut() {
        obj.insert("enabled".to_string(), json!(enabled));
        obj.insert("cadence_valid".to_string(), json!(cadence_valid));
        obj.insert("cadence_label".to_string(), json!(cadence_label));
        obj.insert("interval_secs".to_string(), json!(interval_secs));
        if let Some(tz_mins) = cron_tz {
            obj.insert("tz_offset_mins".to_string(), json!(tz_mins));
            if obj.get("tz").and_then(|v| v.as_str()).is_none() {
                obj.insert("tz".to_string(), json!(format_tz_offset(tz_mins)));
            }
        }
        obj.insert("next_run_ms".to_string(), json!(next));
        obj.insert("due_now".to_string(), json!(due_now));
        obj.insert("missed_windows".to_string(), json!(missed_windows));
        obj.insert("in_seconds".to_string(), json!(in_seconds));
        obj.insert("history_count".to_string(), json!(matching.len()));
        if obj.get("last_receipt_id").is_none() {
            obj.insert("last_receipt_id".to_string(), json!(last_receipt_id));
        }
    }
    out
}

/// Enrich a routine JSON record with SHA-256 receipt chain verification status
/// (`last_run_verified`: `"verified"` | `"drift"` | `null`, `last_run_receipt_sha256`, `last_run_receipt_id`).
pub fn enrich_routine_with_verification(
    routine: &Value,
    now_ms: u64,
    history: &[RoutineRunRecord],
    chain: &[crate::receipts::ChainedReceipt],
    chain_valid: bool,
) -> Value {
    let mut out = enrich_routine_json(routine, now_ms, history);
    let rid = routine["id"].as_str().unwrap_or("");
    let latest = history.iter().find(|h| h.routine_id == rid);
    let last_status = routine["last_status"]
        .as_str()
        .or_else(|| latest.map(|h| h.status.as_str()));
    let last_thread = routine["last_thread_id"]
        .as_str()
        .or_else(|| latest.and_then(|h| h.thread_id.as_deref()));
    let last_rcpt_id = latest
        .and_then(|h| h.receipt_id.as_deref())
        .or_else(|| out["last_receipt_id"].as_str())
        .or(last_thread);

    let matched = last_rcpt_id
        .and_then(|id| crate::receipts::find_receipt(chain, id))
        .or_else(|| last_thread.and_then(|tid| crate::receipts::find_receipt(chain, tid)));

    let run_count = routine["run_count"].as_u64().unwrap_or(0);
    let has_run = run_count > 0 || latest.is_some() || last_status.is_some();

    let (verified_label, sha_val, rcpt_id_val) = if !has_run {
        (Value::Null, Value::Null, Value::Null)
    } else if matches!(
        last_status,
        Some("error" | "failed" | "cancelled" | "no_model")
    ) || !chain_valid
        || matched.is_none()
    {
        (
            json!("drift"),
            matched
                .map(|r| json!(r.sha256))
                .unwrap_or(Value::Null),
            matched
                .map(|r| json!(r.id))
                .unwrap_or(Value::Null),
        )
    } else {
        let r = matched.unwrap();
        (json!("verified"), json!(r.sha256), json!(r.id))
    };

    if let Some(obj) = out.as_object_mut() {
        obj.insert("last_run_verified".to_string(), verified_label);
        obj.insert("last_run_receipt_sha256".to_string(), sha_val);
        obj.insert("last_run_receipt_id".to_string(), rcpt_id_val);
    }
    out
}

/// Summary status snapshot of the background routine scheduler for UI & APIs.
pub fn scheduler_status_json(
    scheduler_enabled: bool,
    tick_secs: u64,
    routines: &[Value],
    history: &[RoutineRunRecord],
    now_ms: u64,
) -> Value {
    let mut active_routines = 0usize;
    let mut paused_routines = 0usize;
    let mut manual_routines = 0usize;
    let mut due_count = 0usize;
    let mut missed_windows_total = 0usize;
    let mut next_due_ms: Option<u64> = None;

    for r in routines {
        let enabled = r["enabled"].as_bool().unwrap_or(true);
        let scheduled = parse_routine_cadence(r)
            .map(|c| c.is_scheduled())
            .unwrap_or(false);

        if !scheduled {
            manual_routines += 1;
        } else if !enabled {
            paused_routines += 1;
        } else {
            active_routines += 1;
            let missed = count_missed_windows(r, now_ms);
            missed_windows_total += missed;
            if let Some(next) = compute_routine_next_run_ms(r, now_ms) {
                if next <= now_ms {
                    due_count += 1;
                }
                next_due_ms = Some(next_due_ms.map(|cur| cur.min(next)).unwrap_or(next));
            }
        }
    }

    let last_run_ms = history.first().map(|h| h.finished_ms);
    json!({
        "enabled": scheduler_enabled,
        "tick_secs": tick_secs,
        "active_routines": active_routines,
        "paused_routines": paused_routines,
        "manual_routines": manual_routines,
        "due_count": due_count,
        "missed_windows_total": missed_windows_total,
        "next_due_ms": next_due_ms,
        "total_runs": history.len(),
        "last_run_ms": last_run_ms,
    })
}
