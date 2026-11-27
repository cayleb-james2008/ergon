// Exact TypeScript mirror of Rust's uncalibrated metering formulas:
// - `src/records.rs`: `round_2`, `round_4`, `compute_minutes_saved`, `compute_billable_value_usd`
// - `src/lib.rs` & `src/pipelines.rs`: hourly rate lookup falling back to `$0/hr` when unassigned.
//
// IMPORTANT: These formulas are static estimates — NOT calibrated to real employee stopwatch runs.
// Do NOT tune constants until `n >= 3` real manual + `n >= 3` real Ergon runs exist per workflow.

export const FORMULA_ESTIMATE_TOOLTIP =
  "Formula estimate — not yet calibrated to stopwatch timings";

export function round2(v: number): number {
  if (!Number.isFinite(v)) return 0;
  const scaled = v * 100;
  const out = Number.isFinite(scaled) ? Math.round(scaled) / 100 : v;
  return out === 0 ? 0 : out;
}

export function round4(v: number): number {
  if (!Number.isFinite(v)) return 0;
  const scaled = v * 10_000;
  const out = Number.isFinite(scaled) ? Math.round(scaled) / 10_000 : v;
  return out === 0 ? 0 : out;
}

/**
 * Exact mirror of `src/records.rs::compute_minutes_saved`.
 * Uncalibrated heuristic estimate from run metadata.
 */
export function computeMinutesSaved(
  finished: boolean,
  answered: boolean,
  okSteps: number,
  filesWritten: number,
  completionTokens: number
): number {
  const steps = Number.isFinite(okSteps) && okSteps > 0 ? Math.floor(okSteps) : 0;
  const files = Number.isFinite(filesWritten) && filesWritten > 0 ? Math.floor(filesWritten) : 0;
  const tokens = Number.isFinite(completionTokens) && completionTokens > 0 ? Math.floor(completionTokens) : 0;

  let raw: number;
  if (!finished) {
    if (steps === 0 && files === 0) {
      raw = 0;
    } else {
      raw = steps * 2.0 + files * 4.0;
    }
  } else if (answered && steps === 0 && files === 0) {
    raw = 2.0 + Math.min(tokens / 250.0, 10.0);
  } else {
    raw = 3.0 + steps * 4.0 + files * 8.0 + Math.min(tokens / 250.0, 15.0);
  }
  return round2(Math.max(0, raw));
}

/**
 * Exact mirror of `src/records.rs::compute_billable_value_usd`.
 */
export function computeBillableValueUsd(minutesSaved: number, hourlyRateUsd: number): number {
  if (
    !Number.isFinite(minutesSaved) ||
    minutesSaved <= 0 ||
    !Number.isFinite(hourlyRateUsd) ||
    hourlyRateUsd <= 0
  ) {
    return 0;
  }
  const val = (minutesSaved / 60.0) * hourlyRateUsd;
  if (!Number.isFinite(val) || val <= 0) return 0;
  return round4(val);
}

/**
 * Exact mirror of `src/lib.rs:3282-3291` and `src/pipelines.rs:2457-2470`:
 * uses active engagement hourly rate if > 0, else client default rate if > 0, else $0/hr.
 */
export function resolveEffectiveHourlyRate(
  engagement: { hourly_rate_usd?: number | null } | null | undefined,
  client: { default_rate_usd?: number | null } | null | undefined
): number {
  if (
    engagement &&
    typeof engagement.hourly_rate_usd === "number" &&
    Number.isFinite(engagement.hourly_rate_usd) &&
    engagement.hourly_rate_usd > 0
  ) {
    return engagement.hourly_rate_usd;
  }
  if (
    client &&
    typeof client.default_rate_usd === "number" &&
    Number.isFinite(client.default_rate_usd) &&
    client.default_rate_usd > 0
  ) {
    return client.default_rate_usd;
  }
  return 0;
}
