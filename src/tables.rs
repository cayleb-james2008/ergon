//! Deterministic structured table loader & query engine for CSV, TSV, XLSX, and JSON.
//!
//! Enforces strict workspace path confinement (CWE-22/59) and exact fixed-point
//! decimal arithmetic (`i128` scaled to 10^9) so financial/ops aggregations
//! never suffer IEEE-754 floating-point drift.

use crate::tools::{atomic_write_bytes, sandbox_path, sandbox_path_resolving};
use crate::{docs, file_history, skills, Config};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

const DEC_SCALE: i128 = 1_000_000_000;
const DEC_PLACES: usize = 9;
const MAX_WHERE_FILTERS: usize = 64;
const MAX_GROUP_BY_COLS: usize = 16;
const MAX_AGGREGATES: usize = 32;

pub(crate) fn ascii_contains_ignore_case(haystack: &str, needle: &str) -> bool {
    let nb = needle.as_bytes();
    if nb.is_empty() {
        return true;
    }
    let hb = haystack.as_bytes();
    if nb.len() > hb.len() {
        return false;
    }
    hb.windows(nb.len()).any(|w| w.eq_ignore_ascii_case(nb))
}

fn read_utf8_table_text(full: &Path, base: &str, label: &str) -> Result<String, String> {
    let bytes = std::fs::read(full).map_err(|e| format!("could not read {label} {base}: {e}"))?;
    if bytes.contains(&0) {
        return Err(format!(
            "{label} file {base} appears to be binary (contains 0x00 bytes)"
        ));
    }
    String::from_utf8(bytes)
        .map_err(|_| format!("{label} file {base} is not valid UTF-8"))
}

#[derive(Clone, Debug)]
pub struct LoadedTable {
    pub path: String,
    pub sheet: Option<String>,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct TableQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,
    pub matched_rows: usize,
    pub saved_path: Option<String>,
}

impl TableQueryResult {
    pub fn to_json(&self) -> Value {
        let mut v = json!({
            "columns": self.columns,
            "rows": self.rows,
            "total_rows": self.total_rows,
            "matched_rows": self.matched_rows,
        });
        if let Some(sp) = &self.saved_path {
            v["saved_path"] = json!(sp);
        }
        v
    }
}

#[derive(Clone, Debug)]
pub struct VaultAggregateResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub source_files: Vec<String>,
    pub total_rows: usize,
    pub matched_rows: usize,
    pub saved_path: Option<String>,
}

impl VaultAggregateResult {
    pub fn to_json(&self) -> Value {
        let mut v = json!({
            "ok": true,
            "columns": self.columns,
            "rows": self.rows,
            "source_files": self.source_files,
            "total_rows": self.total_rows,
            "matched_rows": self.matched_rows,
            "row_count": self.rows.len(),
        });
        if let Some(sp) = &self.saved_path {
            v["saved_path"] = json!(sp);
        }
        v
    }
}

/// Strictly validate a workspace-relative path against traversal, absolute
/// paths, Windows drive letters, NUL/control chars, encoded `..`, and dotfiles.
pub fn validate_workspace_rel_path(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("path is required".to_string());
    }
    if trimmed.len() > 240 || trimmed.split('/').any(|s| s.len() > 200) {
        return Err("path is too long".to_string());
    }
    let low = trimmed.to_ascii_lowercase();
    if trimmed.contains('\0')
        || trimmed.contains('\\')
        || trimmed.contains(':')
        || trimmed.chars().any(|c| c.is_control())
        || trimmed.starts_with('/')
        || low.contains("%2e")
        || low.contains("%2f")
        || low.contains("%5c")
        || low.contains("%00")
        || low.contains("%25")
        || low.contains("connector_secrets.json")
        || low.contains("secrets.enc.json")
        || low.contains(".vault.key")
        || low.contains("oauth.json")
        || trimmed
            .split('/')
            .any(|seg| seg.is_empty() || seg == "." || seg == ".." || seg.starts_with('.'))
    {
        return Err(format!("path escapes the workspace: {trimmed}"));
    }
    Ok(trimmed.to_string())
}

/// Load a workspace `.csv`, `.tsv`, `.xlsx`/`.xlsm` (`path#SheetName`), or `.json`
/// array-of-objects table with strict workspace path confinement.
pub fn load_table(
    cfg: &Config,
    raw_path: &str,
    sheet_override: Option<&str>,
) -> Result<LoadedTable, String> {
    let trimmed = raw_path.trim();
    let (base_raw, frag_sheet) = docs::split_sheet(trimmed);
    let base = validate_workspace_rel_path(base_raw)?;
    let full = sandbox_path(&cfg.workspace_dir, &base)?;
    if full == cfg.workspace_dir || full.is_dir() {
        return Err(format!("path must name a file inside the workspace: {base}"));
    }
    let md = std::fs::metadata(&full).map_err(|_| format!("table file not found: {base}"))?;
    if md.len() > docs::MAX_DOC_BYTES {
        return Err(format!(
            "table file {base} exceeds maximum size of {} MB",
            docs::MAX_DOC_BYTES / (1024 * 1024)
        ));
    }

    let wanted_sheet = sheet_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or(frag_sheet)
        .map(str::to_string);

    let ext = base.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    let (mut raw_rows, resolved_sheet) = match ext.as_str() {
        "csv" => {
            let text = read_utf8_table_text(&full, &base, "CSV")?;
            (skills::parse_csv_rows(&text), None)
        }
        "tsv" => {
            let text = read_utf8_table_text(&full, &base, "TSV")?;
            let clean = text.strip_prefix('\u{feff}').unwrap_or(&text);
            let rows: Vec<Vec<String>> = clean
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| {
                    l.trim_end_matches('\r')
                        .split('\t')
                        .map(|c| c.to_string())
                        .collect()
                })
                .collect();
            (rows, None)
        }
        "xlsx" | "xlsm" => {
            let bytes = std::fs::read(&full)
                .map_err(|e| format!("could not read workbook {base}: {e}"))?;
            let sheets = docs::xlsx_sheets(&bytes)?;
            let selected = match &wanted_sheet {
                Some(name) => sheets
                    .iter()
                    .find(|s| s.name == *name)
                    .or_else(|| sheets.iter().find(|s| s.name.eq_ignore_ascii_case(name)))
                    .ok_or_else(|| {
                        let avail: Vec<&str> = sheets.iter().map(|s| s.name.as_str()).collect();
                        format!(
                            "no sheet named '{}' in {base} (available: {})",
                            name,
                            avail.join(", ")
                        )
                    })?,
                None => sheets
                    .first()
                    .ok_or_else(|| format!("workbook {base} has no readable sheets"))?,
            };
            (selected.rows.clone(), Some(selected.name.clone()))
        }
        "json" => {
            let text = read_utf8_table_text(&full, &base, "JSON")?;
            let clean = text.strip_prefix('\u{feff}').unwrap_or(&text).trim();
            let parsed: Value = serde_json::from_str(clean)
                .map_err(|e| format!("invalid JSON table in {base}: {e}"))?;
            let rows = skills::parse_json_table_rows(clean);
            if rows.is_empty() {
                if let Value::Array(arr) = &parsed {
                    if arr.is_empty() {
                        return Ok(LoadedTable {
                            path: base,
                            sheet: None,
                            columns: Vec::new(),
                            rows: Vec::new(),
                        });
                    }
                }
                return Err(format!(
                    "JSON file {base} is not a structured array-of-objects table"
                ));
            }
            (rows, None)
        }
        _ => {
            return Err(format!(
                "unsupported table format for {base} (supported: .csv, .tsv, .xlsx, .json)"
            ));
        }
    };

    if raw_rows.is_empty() {
        return Ok(LoadedTable {
            path: base,
            sheet: resolved_sheet,
            columns: Vec::new(),
            rows: Vec::new(),
        });
    }

    let mut columns = raw_rows.remove(0);
    let max_row_width = raw_rows.iter().map(Vec::len).max().unwrap_or(0);
    let width = columns.len().max(max_row_width);
    while columns.len() < width {
        let mut idx = columns.len() + 1;
        let mut name = format!("col_{idx}");
        while columns.iter().any(|c| c == &name) {
            idx += 1;
            name = format!("col_{idx}");
        }
        columns.push(name);
    }
    let mut rows = raw_rows;
    for r in &mut rows {
        r.resize(width, String::new());
    }

    Ok(LoadedTable {
        path: base,
        sheet: resolved_sheet,
        columns,
        rows,
    })
}

/// Parse an exact fixed-point decimal (`i128` * 10^9) from plain numbers,
/// currency strings (`$1,250.50`), and accounting negatives (`(42.00)`).
/// Rejects leading-zero integer identifiers (`02139`) to match `docs::typed_cell`.
pub fn parse_decimal(raw: &str) -> Option<i128> {
    let s = raw.trim();
    if s.is_empty() || s.len() > 48 {
        return None;
    }
    let (neg_paren, body) = match s.strip_prefix('(').and_then(|x| x.strip_suffix(')')) {
        Some(inner) => (true, inner.trim()),
        None => (false, s),
    };
    let body = body.trim();
    if body.is_empty() {
        return None;
    }

    // Extract optional leading sign and currency markers cleanly.
    let mut neg = neg_paren;
    let mut rest = body;
    if let Some(r) = rest.strip_prefix('-') {
        neg = !neg;
        rest = r.trim_start();
    } else if let Some(r) = rest.strip_prefix('+') {
        rest = r.trim_start();
    }
    if let Some(r) = rest.strip_prefix(['$', '€', '£']) {
        rest = r.trim_start();
    }
    if let Some(r) = rest.strip_prefix('-') {
        neg = !neg;
        rest = r.trim_start();
    }
    if rest.is_empty() {
        return None;
    }

    // Validate comma grouping when commas are present (e.g. `1,234.56`).
    if rest.contains(',') {
        let int_part = rest.split('.').next().unwrap_or("");
        let mut groups = int_part.split(',');
        let first = groups.next().unwrap_or("");
        if first.is_empty() || first.len() > 3 || !first.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        for g in groups {
            if g.len() != 3 || !g.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
        }
    }

    let cleaned: String = rest.chars().filter(|&c| c != ',').collect();
    if cleaned.is_empty()
        || !cleaned.chars().all(|c| c.is_ascii_digit() || c == '.')
        || cleaned.chars().filter(|&c| c == '.').count() > 1
    {
        return None;
    }

    // Preserve leading-zero identifiers like ZIP codes ("02139") as strings.
    if cleaned.len() > 1 && cleaned.starts_with('0') && !cleaned.starts_with("0.") {
        return None;
    }

    let (int_str, frac_str) = match cleaned.split_once('.') {
        Some((a, b)) => (a, b),
        None => (cleaned.as_str(), ""),
    };
    if int_str.is_empty() && frac_str.is_empty() {
        return None;
    }
    if int_str.len() > 20 {
        return None;
    }

    let int_val: i128 = if int_str.is_empty() {
        0
    } else {
        int_str.parse().ok()?
    };

    let mut frac_val: i128 = 0;
    let mut weight = DEC_SCALE / 10;
    for b in frac_str.bytes().take(DEC_PLACES) {
        frac_val += ((b - b'0') as i128) * weight;
        weight /= 10;
    }
    // Round half-up on the 10th fractional digit if present.
    if frac_str.len() > DEC_PLACES && frac_str.as_bytes()[DEC_PLACES] >= b'5' {
        frac_val += 1;
    }

    let combined = int_val.checked_mul(DEC_SCALE)?.checked_add(frac_val)?;
    Some(if neg && combined != 0 { -combined } else { combined })
}

pub fn format_decimal(val: i128, max_frac_digits: usize) -> String {
    if val == 0 {
        return "0".to_string();
    }
    let neg = val < 0;
    let abs = val.unsigned_abs();
    let scale = DEC_SCALE as u128;
    let int_part = abs / scale;
    let frac_part = abs % scale;
    if frac_part == 0 || max_frac_digits == 0 {
        return if neg {
            format!("-{int_part}")
        } else {
            int_part.to_string()
        };
    }
    let mut frac_str = format!("{frac_part:09}");
    let keep = max_frac_digits.min(DEC_PLACES);
    frac_str.truncate(keep);
    let trimmed = frac_str.trim_end_matches('0');
    if trimmed.is_empty() {
        if neg {
            format!("-{int_part}")
        } else {
            int_part.to_string()
        }
    } else if neg {
        format!("-{int_part}.{trimmed}")
    } else {
        format!("{int_part}.{trimmed}")
    }
}

fn find_col(columns: &[String], name: &str) -> Result<usize, String> {
    let target = name.trim();
    if target.is_empty() {
        return Err("column name cannot be empty".to_string());
    }
    columns
        .iter()
        .position(|c| c == target)
        .or_else(|| columns.iter().position(|c| c.eq_ignore_ascii_case(target)))
        .ok_or_else(|| {
            format!(
                "unknown column '{target}' (available: {})",
                columns.join(", ")
            )
        })
}

#[derive(Clone, Copy)]
enum FilterOp {
    Eq,
    Neq,
    Contains,
    Gt,
    Gte,
    Lt,
    Lte,
}

struct CompiledFilter {
    col_idx: usize,
    op: FilterOp,
    raw_val: String,
    lower_val: String,
    dec_val: Option<i128>,
}

impl CompiledFilter {
    fn matches(&self, row: &[String]) -> bool {
        let cell = row.get(self.col_idx).map(String::as_str).unwrap_or("");
        let cell_t = cell.trim();
        let val_t = self.raw_val.trim();
        match self.op {
            FilterOp::Eq => {
                if cell_t == val_t {
                    return true;
                }
                match (parse_decimal(cell_t), self.dec_val) {
                    (Some(a), Some(b)) => a == b,
                    _ => false,
                }
            }
            FilterOp::Neq => {
                if cell_t == val_t {
                    return false;
                }
                match (parse_decimal(cell_t), self.dec_val) {
                    (Some(a), Some(b)) => a != b,
                    _ => true,
                }
            }
            FilterOp::Contains => ascii_contains_ignore_case(cell, &self.lower_val),
            FilterOp::Gt | FilterOp::Gte | FilterOp::Lt | FilterOp::Lte => {
                let ord = match (parse_decimal(cell_t), self.dec_val) {
                    (Some(a), Some(b)) => a.cmp(&b),
                    _ => cell_t.cmp(val_t),
                };
                match self.op {
                    FilterOp::Gt => ord.is_gt(),
                    FilterOp::Gte => ord.is_ge(),
                    FilterOp::Lt => ord.is_lt(),
                    FilterOp::Lte => ord.is_le(),
                    _ => unreachable!(),
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum AggFn {
    Sum,
    Avg,
    Min,
    Max,
    Count,
}

struct AggSpec {
    col_idx: Option<usize>,
    col_name: String,
    func: AggFn,
    alias: String,
}

#[derive(Clone)]
enum MinMaxVal {
    Num(i128),
    Text(String),
}

struct AggState {
    count_all: usize,
    count_non_empty: usize,
    sum: i128,
    min: Option<MinMaxVal>,
    max: Option<MinMaxVal>,
}

impl AggState {
    fn new() -> Self {
        Self {
            count_all: 0,
            count_non_empty: 0,
            sum: 0,
            min: None,
            max: None,
        }
    }

    fn update(&mut self, func: AggFn, cell: Option<&str>, col_name: &str) -> Result<(), String> {
        self.count_all += 1;
        let Some(raw) = cell else {
            return Ok(());
        };
        let t = raw.trim();
        if t.is_empty() {
            return Ok(());
        }
        self.count_non_empty += 1;
        match func {
            AggFn::Count => Ok(()),
            AggFn::Sum | AggFn::Avg => {
                let d = parse_decimal(t).ok_or_else(|| {
                    format!("cannot aggregate non-numeric value '{t}' in column '{col_name}'")
                })?;
                self.sum = self
                    .sum
                    .checked_add(d)
                    .ok_or_else(|| format!("numeric overflow aggregating column '{col_name}'"))?;
                Ok(())
            }
            AggFn::Min => {
                let cand = match parse_decimal(t) {
                    Some(d) => MinMaxVal::Num(d),
                    None => MinMaxVal::Text(t.to_string()),
                };
                self.min = Some(match self.min.take() {
                    None => cand,
                    Some(cur) => match (cur, cand) {
                        (MinMaxVal::Num(a), MinMaxVal::Num(b)) => MinMaxVal::Num(a.min(b)),
                        (MinMaxVal::Text(a), MinMaxVal::Text(b)) => {
                            MinMaxVal::Text(if b < a { b } else { a })
                        }
                        (MinMaxVal::Num(a), MinMaxVal::Text(b)) => {
                            let s = format_decimal(a, DEC_PLACES);
                            MinMaxVal::Text(if b < s { b } else { s })
                        }
                        (MinMaxVal::Text(a), MinMaxVal::Num(b)) => {
                            let s = format_decimal(b, DEC_PLACES);
                            MinMaxVal::Text(if s < a { s } else { a })
                        }
                    },
                });
                Ok(())
            }
            AggFn::Max => {
                let cand = match parse_decimal(t) {
                    Some(d) => MinMaxVal::Num(d),
                    None => MinMaxVal::Text(t.to_string()),
                };
                self.max = Some(match self.max.take() {
                    None => cand,
                    Some(cur) => match (cur, cand) {
                        (MinMaxVal::Num(a), MinMaxVal::Num(b)) => MinMaxVal::Num(a.max(b)),
                        (MinMaxVal::Text(a), MinMaxVal::Text(b)) => {
                            MinMaxVal::Text(if b > a { b } else { a })
                        }
                        (MinMaxVal::Num(a), MinMaxVal::Text(b)) => {
                            let s = format_decimal(a, DEC_PLACES);
                            MinMaxVal::Text(if b > s { b } else { s })
                        }
                        (MinMaxVal::Text(a), MinMaxVal::Num(b)) => {
                            let s = format_decimal(b, DEC_PLACES);
                            MinMaxVal::Text(if s > a { s } else { a })
                        }
                    },
                });
                Ok(())
            }
        }
    }

    fn finish(&self, func: AggFn, star_count: bool) -> String {
        match func {
            AggFn::Count => {
                if star_count {
                    self.count_all.to_string()
                } else {
                    self.count_non_empty.to_string()
                }
            }
            AggFn::Sum => {
                if self.count_non_empty == 0 {
                    "0".to_string()
                } else {
                    format_decimal(self.sum, DEC_PLACES)
                }
            }
            AggFn::Avg => {
                if self.count_non_empty == 0 {
                    String::new()
                } else {
                    let n = self.count_non_empty as i128;
                    let half = n / 2;
                    let q = if self.sum >= 0 {
                        (self.sum + half) / n
                    } else {
                        (self.sum - half) / n
                    };
                    format_decimal(q, 6)
                }
            }
            AggFn::Min => match &self.min {
                Some(MinMaxVal::Num(d)) => format_decimal(*d, DEC_PLACES),
                Some(MinMaxVal::Text(s)) => s.clone(),
                None => String::new(),
            },
            AggFn::Max => match &self.max {
                Some(MinMaxVal::Num(d)) => format_decimal(*d, DEC_PLACES),
                Some(MinMaxVal::Text(s)) => s.clone(),
                None => String::new(),
            },
        }
    }
}

fn json_scalar_to_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn normalize_table_rows(mut raw_rows: Vec<Vec<String>>) -> (Vec<String>, Vec<Vec<String>>) {
    if raw_rows.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let mut columns = raw_rows.remove(0);
    let max_row_width = raw_rows.iter().map(Vec::len).max().unwrap_or(0);
    let width = columns.len().max(max_row_width);
    while columns.len() < width {
        let mut idx = columns.len() + 1;
        let mut name = format!("col_{idx}");
        while columns.iter().any(|c| c == &name) {
            idx += 1;
            name = format!("col_{idx}");
        }
        columns.push(name);
    }
    for r in &mut raw_rows {
        r.resize(width, String::new());
    }
    (columns, raw_rows)
}

fn run_table_pipeline(
    cfg: &Config,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    args: &Value,
    revision_op: &str,
) -> Result<TableQueryResult, String> {
    let total_rows = rows.len();

    // Compile `where` filters.
    let mut filters: Vec<CompiledFilter> = Vec::new();
    if let Some(arr) = args["where"].as_array() {
        if arr.len() > MAX_WHERE_FILTERS {
            return Err(format!("too many where filters (max {MAX_WHERE_FILTERS})"));
        }
        for item in arr {
            let col = item["col"]
                .as_str()
                .ok_or("where filter requires 'col'")?;
            let op_str = item["op"]
                .as_str()
                .ok_or("where filter requires 'op'")?
                .trim()
                .to_ascii_lowercase();
            let op = match op_str.as_str() {
                "eq" | "=" | "==" => FilterOp::Eq,
                "neq" | "!=" | "<>" => FilterOp::Neq,
                "contains" => FilterOp::Contains,
                "gt" | ">" => FilterOp::Gt,
                "gte" | ">=" => FilterOp::Gte,
                "lt" | "<" => FilterOp::Lt,
                "lte" | "<=" => FilterOp::Lte,
                other => {
                    return Err(format!(
                        "unknown where operator '{other}' (use eq, neq, contains, gt, gte, lt, lte)"
                    ))
                }
            };
            let raw_val = json_scalar_to_string(&item["value"]);
            let col_idx = find_col(&columns, col)?;
            let dec_val = parse_decimal(&raw_val);
            filters.push(CompiledFilter {
                col_idx,
                op,
                lower_val: raw_val.to_ascii_lowercase(),
                raw_val,
                dec_val,
            });
        }
    }

    // Filter rows.
    let matched_indices: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| filters.iter().all(|f| f.matches(r)))
        .map(|(i, _)| i)
        .collect();
    let matched_rows = matched_indices.len();

    // Compile `group_by` and `aggregates`.
    let group_cols_raw: Vec<String> = args["group_by"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::trim))
                .filter(|s| !s.is_empty())
                .take(MAX_GROUP_BY_COLS)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let mut group_indices: Vec<usize> = Vec::with_capacity(group_cols_raw.len());
    let mut group_names: Vec<String> = Vec::with_capacity(group_cols_raw.len());
    for g in &group_cols_raw {
        let idx = find_col(&columns, g)?;
        group_indices.push(idx);
        group_names.push(columns[idx].clone());
    }

    let mut agg_specs: Vec<AggSpec> = Vec::new();
    if let Some(arr) = args["aggregates"].as_array() {
        if arr.len() > MAX_AGGREGATES {
            return Err(format!("too many aggregates (max {MAX_AGGREGATES})"));
        }
        for item in arr {
            let fn_str = item["fn"]
                .as_str()
                .ok_or("aggregate requires 'fn'")?
                .trim()
                .to_ascii_lowercase();
            let func = match fn_str.as_str() {
                "sum" => AggFn::Sum,
                "avg" | "mean" => AggFn::Avg,
                "min" => AggFn::Min,
                "max" => AggFn::Max,
                "count" => AggFn::Count,
                other => {
                    return Err(format!(
                        "unknown aggregate fn '{other}' (use sum, avg, min, max, count)"
                    ))
                }
            };
            let col_raw = item["col"].as_str().unwrap_or("").trim();
            let col_idx = if matches!(func, AggFn::Count) && (col_raw.is_empty() || col_raw == "*")
            {
                None
            } else {
                Some(find_col(&columns, col_raw)?)
            };
            let alias = item["as"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| {
                    if col_raw.is_empty() || col_raw == "*" {
                        fn_str.clone()
                    } else {
                        format!("{fn_str}_{col_raw}")
                    }
                });
            agg_specs.push(AggSpec {
                col_idx,
                col_name: col_raw.to_string(),
                func,
                alias,
            });
        }
    }

    let (out_columns, mut out_rows) = if group_indices.is_empty() && agg_specs.is_empty() {
        let filtered_rows: Vec<Vec<String>> = matched_indices
            .iter()
            .map(|&i| rows[i].clone())
            .collect();
        (columns, filtered_rows)
    } else {
        let mut cols = group_names;
        for a in &agg_specs {
            cols.push(a.alias.clone());
        }

        let mut group_map: HashMap<Vec<String>, usize> = HashMap::new();
        let mut group_keys: Vec<Vec<String>> = Vec::new();
        let mut group_aggs: Vec<Vec<AggState>> = Vec::new();

        // If `group_by` is empty and `aggregates` is non-empty, emit 1 grand-total row even if 0 rows matched.
        if group_indices.is_empty() {
            group_keys.push(Vec::new());
            group_aggs.push((0..agg_specs.len()).map(|_| AggState::new()).collect());
        }

        for idx in matched_indices {
            let row = &rows[idx];
            let gi = if group_indices.is_empty() {
                0
            } else {
                let key: Vec<String> = group_indices
                    .iter()
                    .map(|&ci| row.get(ci).cloned().unwrap_or_default())
                    .collect();
                if let Some(&existing) = group_map.get(&key) {
                    existing
                } else {
                    let next_i = group_keys.len();
                    group_map.insert(key.clone(), next_i);
                    group_keys.push(key);
                    group_aggs.push((0..agg_specs.len()).map(|_| AggState::new()).collect());
                    next_i
                }
            };
            let states = &mut group_aggs[gi];
            for (ai, spec) in agg_specs.iter().enumerate() {
                let cell = spec.col_idx.and_then(|ci| row.get(ci).map(String::as_str));
                states[ai].update(spec.func, cell, &spec.col_name)?;
            }
        }

        let mut aggregated_rows = Vec::with_capacity(group_keys.len());
        for (gi, mut key) in group_keys.into_iter().enumerate() {
            for (ai, spec) in agg_specs.iter().enumerate() {
                key.push(group_aggs[gi][ai].finish(spec.func, spec.col_idx.is_none()));
            }
            aggregated_rows.push(key);
        }
        (cols, aggregated_rows)
    };

    // Sort output rows if `sort` is provided.
    if let Some(sort_obj) = args["sort"].as_object() {
        if let Some(sort_col) = sort_obj.get("col").and_then(|v| v.as_str()) {
            let sort_idx = find_col(&out_columns, sort_col)?;
            let desc = sort_obj
                .get("dir")
                .and_then(|v| v.as_str())
                .map(|d| d.trim().eq_ignore_ascii_case("desc"))
                .unwrap_or(false);
            out_rows.sort_by(|a, b| {
                let va = a.get(sort_idx).map(String::as_str).unwrap_or("");
                let vb = b.get(sort_idx).map(String::as_str).unwrap_or("");
                let ord = match (parse_decimal(va), parse_decimal(vb)) {
                    (Some(da), Some(db)) => da.cmp(&db),
                    _ => va
                        .to_ascii_lowercase()
                        .cmp(&vb.to_ascii_lowercase())
                        .then_with(|| va.cmp(vb)),
                };
                if desc {
                    ord.reverse()
                } else {
                    ord
                }
            });
        }
    }

    // Apply `limit`.
    if let Some(lim) = args["limit"].as_u64() {
        out_rows.truncate(lim as usize);
    }

    // Optional `save_as` CSV emission into the workspace.
    let saved_path = if let Some(raw_save) = args["save_as"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let save_rel = validate_workspace_rel_path(raw_save)?;
        let unresolved = sandbox_path_resolving(&cfg.workspace_dir, &save_rel, false)?;
        if sandbox_path_resolving(&cfg.workspace_dir, &save_rel, true).is_err()
            || unresolved
                .symlink_metadata()
                .is_ok_and(|md| md.file_type().is_symlink())
        {
            return Err(format!("save_as cannot overwrite symlink: {save_rel}"));
        }
        if unresolved.is_dir() {
            return Err(format!(
                "save_as must name a file, not a directory: {save_rel}"
            ));
        }
        let mut csv = String::with_capacity((out_rows.len() + 1) * 48);
        csv.push_str(&docs::csv_line(&out_columns));
        csv.push('\n');
        for r in &out_rows {
            csv.push_str(&docs::csv_line(r));
            csv.push('\n');
        }
        let _ = file_history::record_revision(&cfg.data_dir, &cfg.workspace_dir, &save_rel, revision_op);
        atomic_write_bytes(&unresolved, csv.as_bytes())
            .map_err(|e| format!("save_as {save_rel}: {e}"))?;
        Some(save_rel)
    } else {
        None
    };

    Ok(TableQueryResult {
        columns: out_columns,
        rows: out_rows,
        total_rows,
        matched_rows,
        saved_path,
    })
}

/// Execute a structured query (`where`, `group_by`, `aggregates`, `sort`,
/// `limit`, `save_as`) against a workspace CSV/TSV/XLSX/JSON table.
pub fn execute_query(cfg: &Config, args: &Value) -> Result<TableQueryResult, String> {
    let raw_path = args["path"]
        .as_str()
        .ok_or("query_table needs args.path")?;
    let sheet_override = args["sheet"].as_str();
    let table = load_table(cfg, raw_path, sheet_override)?;
    run_table_pipeline(cfg, table.columns, table.rows, args, "query_table")
}

const MAX_GLOB_BRACE_PATTERNS: usize = 64;

fn check_glob_segments_safe(pat: &str, raw_display: &str) -> Result<(), String> {
    if pat.is_empty()
        || pat.starts_with('/')
        || pat.split('/').any(|seg| {
            seg.is_empty() || seg == "." || seg == ".." || seg.starts_with('.')
        })
    {
        return Err(format!("glob pattern escapes the workspace: {raw_display}"));
    }
    Ok(())
}

fn validate_glob_pattern(raw: &str) -> Result<String, String> {
    let mut trimmed = raw.trim().to_string();
    if trimmed.is_empty() {
        return Err("aggregate_vault needs args.glob".to_string());
    }
    while let Some(rest) = trimmed.strip_prefix("./") {
        trimmed = rest.to_string();
    }
    if trimmed.is_empty() {
        return Err("aggregate_vault needs args.glob".to_string());
    }
    if trimmed.len() > 240 {
        return Err("malformed glob pattern: exceeds 240 characters".to_string());
    }
    let low = trimmed.to_ascii_lowercase();
    if trimmed.contains('\0')
        || trimmed.contains('\\')
        || trimmed.contains(':')
        || trimmed.chars().any(|c| c.is_control())
        || trimmed.starts_with('/')
        || low.contains("%2e")
        || low.contains("%2f")
        || low.contains("%5c")
        || low.contains("%00")
        || low.contains("%25")
        || low.contains("connector_secrets.json")
        || low.contains("secrets.enc.json")
        || low.contains(".vault.key")
        || low.contains("oauth.json")
    {
        return Err(format!("glob pattern escapes the workspace: {trimmed}"));
    }

    // Validate brace syntax (balanced, non-nested, non-empty alternatives).
    let mut depth = 0usize;
    let mut cur_alt_nonempty = false;
    for ch in trimmed.chars() {
        match ch {
            '{' => {
                if depth > 0 {
                    return Err("malformed glob pattern: nested '{' braces are not supported".to_string());
                }
                depth = 1;
                cur_alt_nonempty = false;
            }
            ',' if depth == 1 => {
                if !cur_alt_nonempty {
                    return Err("malformed glob pattern: empty brace alternative".to_string());
                }
                cur_alt_nonempty = false;
            }
            '}' => {
                if depth == 0 {
                    return Err("malformed glob pattern: unmatched '}'".to_string());
                }
                if !cur_alt_nonempty {
                    return Err("malformed glob pattern: empty brace alternative".to_string());
                }
                depth = 0;
            }
            _ => {
                if depth == 1 {
                    cur_alt_nonempty = true;
                }
            }
        }
    }
    if depth != 0 {
        return Err("malformed glob pattern: unclosed '{'".to_string());
    }

    check_glob_segments_safe(&trimmed, &trimmed)?;
    let expanded = expand_glob_braces_bounded(&trimmed)?;
    for pat in &expanded {
        check_glob_segments_safe(pat, &trimmed)?;
    }
    Ok(trimmed)
}

fn expand_glob_braces_bounded(pattern: &str) -> Result<Vec<String>, String> {
    if let Some(open) = pattern.find('{') {
        if let Some(rel_close) = pattern[open + 1..].find('}') {
            let close = open + 1 + rel_close;
            let prefix = &pattern[..open];
            let suffix = &pattern[close + 1..];
            let mut out = Vec::new();
            for alt in pattern[open + 1..close].split(',') {
                let sub_patterns = expand_glob_braces_bounded(&format!("{prefix}{alt}{suffix}"))?;
                if out.len() + sub_patterns.len() > MAX_GLOB_BRACE_PATTERNS {
                    return Err(format!(
                        "malformed glob pattern: brace expansion exceeds {MAX_GLOB_BRACE_PATTERNS} patterns"
                    ));
                }
                out.extend(sub_patterns);
            }
            return Ok(out);
        }
    }
    Ok(vec![pattern.to_string()])
}

fn expand_glob_braces(pattern: &str) -> Vec<String> {
    expand_glob_braces_bounded(pattern).unwrap_or_else(|_| vec![pattern.to_string()])
}

fn glob_seg_match(pat: &[char], val: &[char]) -> bool {
    let (mut pi, mut vi) = (0usize, 0usize);
    let mut star_pi: Option<usize> = None;
    let mut star_vi = 0usize;
    while vi < val.len() {
        if pi < pat.len() && (pat[pi] == '?' || pat[pi] == val[vi]) {
            pi += 1;
            vi += 1;
        } else if pi < pat.len() && pat[pi] == '*' {
            star_pi = Some(pi);
            star_vi = vi;
            pi += 1;
        } else if let Some(sp) = star_pi {
            pi = sp + 1;
            star_vi += 1;
            vi = star_vi;
        } else {
            return false;
        }
    }
    while pi < pat.len() && pat[pi] == '*' {
        pi += 1;
    }
    pi == pat.len()
}

fn glob_segs_match(pat_segs: &[&str], path_segs: &[&str]) -> bool {
    if pat_segs.is_empty() {
        return path_segs.is_empty();
    }
    if pat_segs[0] == "**" {
        if glob_segs_match(&pat_segs[1..], path_segs) {
            return true;
        }
        return !path_segs.is_empty() && glob_segs_match(pat_segs, &path_segs[1..]);
    }
    if path_segs.is_empty() {
        return false;
    }
    let p_chars: Vec<char> = pat_segs[0].chars().collect();
    let v_chars: Vec<char> = path_segs[0].chars().collect();
    glob_seg_match(&p_chars, &v_chars) && glob_segs_match(&pat_segs[1..], &path_segs[1..])
}

fn glob_match_any(patterns: &[String], rel_path: &str) -> bool {
    let path_segs: Vec<&str> = rel_path.split('/').collect();
    patterns.iter().any(|pat| {
        let pat_segs: Vec<&str> = pat.split('/').collect();
        glob_segs_match(&pat_segs, &path_segs)
    })
}

fn collect_workspace_table_files(
    root: &Path,
    dir: &Path,
    patterns: &[String],
    out: &mut Vec<String>,
) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        if out.len() >= 500 {
            return;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.')
            || name.eq_ignore_ascii_case("connector_secrets.json")
            || name.eq_ignore_ascii_case("secrets.enc.json")
            || name.eq_ignore_ascii_case("oauth.json")
        {
            continue;
        }
        let path = entry.path();
        let Ok(md) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if md.file_type().is_symlink() {
            continue;
        }
        if md.is_dir() {
            collect_workspace_table_files(root, &path, patterns, out);
            continue;
        }
        if !md.is_file() {
            continue;
        }
        let Ok(rel_path) = path.strip_prefix(root) else {
            continue;
        };
        let rel = rel_path.to_string_lossy().replace('\\', "/");
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !matches!(ext.as_str(), "csv" | "tsv" | "xlsx" | "xlsm" | "json") {
            continue;
        }
        if glob_match_any(patterns, &rel) {
            out.push(rel);
        }
    }
}

fn args_mentions_col(args: &Value, target: &str) -> bool {
    let check_str = |s: &str| s.trim().eq_ignore_ascii_case(target);
    if args["where"]
        .as_array()
        .is_some_and(|arr| arr.iter().any(|w| w["col"].as_str().is_some_and(check_str)))
    {
        return true;
    }
    if args["group_by"]
        .as_array()
        .is_some_and(|arr| arr.iter().any(|g| g.as_str().is_some_and(check_str)))
    {
        return true;
    }
    if args["aggregates"]
        .as_array()
        .is_some_and(|arr| arr.iter().any(|a| a["col"].as_str().is_some_and(check_str)))
    {
        return true;
    }
    if args["sort"]["col"].as_str().is_some_and(check_str) {
        return true;
    }
    false
}

/// Execute deterministic exact-decimal multi-file aggregation across matching
/// CSV, TSV, and XLSX files/sheets inside the sandboxed workspace.
pub fn execute_vault_aggregate(cfg: &Config, args: &Value) -> Result<VaultAggregateResult, String> {
    const MAX_COMBINED_ROWS: usize = 500_000;
    let raw_glob = args["glob"]
        .as_str()
        .ok_or("aggregate_vault needs args.glob")?;
    let clean_glob = validate_glob_pattern(raw_glob)?;
    let patterns = expand_glob_braces(&clean_glob);
    let wanted_sheet = args["sheet"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "*");

    let mut matched_files = Vec::new();
    collect_workspace_table_files(&cfg.workspace_dir, &cfg.workspace_dir, &patterns, &mut matched_files);
    matched_files.sort();
    if matched_files.is_empty() {
        return Err(format!(
            "no CSV, TSV, or XLSX files matched glob '{clean_glob}'"
        ));
    }

    let mut loaded_tables: Vec<LoadedTable> = Vec::new();
    for rel in &matched_files {
        let full = sandbox_path(&cfg.workspace_dir, rel)?;
        let md = std::fs::metadata(&full).map_err(|_| format!("table file not found: {rel}"))?;
        if md.len() > docs::MAX_DOC_BYTES {
            return Err(format!(
                "table file {rel} exceeds maximum size of {} MB",
                docs::MAX_DOC_BYTES / (1024 * 1024)
            ));
        }
        let ext = rel.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        if ext == "xlsx" || ext == "xlsm" {
            let bytes = std::fs::read(&full)
                .map_err(|e| format!("could not read workbook {rel}: {e}"))?;
            let sheets = docs::xlsx_sheets(&bytes)?;
            if let Some(target_sheet) = wanted_sheet {
                let selected = sheets
                    .iter()
                    .find(|s| s.name == target_sheet)
                    .or_else(|| sheets.iter().find(|s| s.name.eq_ignore_ascii_case(target_sheet)))
                    .ok_or_else(|| {
                        let avail: Vec<&str> = sheets.iter().map(|s| s.name.as_str()).collect();
                        format!(
                            "no sheet named '{}' in {rel} (available: {})",
                            target_sheet,
                            avail.join(", ")
                        )
                    })?;
                let (columns, rows) = normalize_table_rows(selected.rows.clone());
                loaded_tables.push(LoadedTable {
                    path: rel.clone(),
                    sheet: Some(selected.name.clone()),
                    columns,
                    rows,
                });
            } else {
                for s in sheets {
                    if s.rows.is_empty() {
                        continue;
                    }
                    let (columns, rows) = normalize_table_rows(s.rows);
                    loaded_tables.push(LoadedTable {
                        path: rel.clone(),
                        sheet: Some(s.name),
                        columns,
                        rows,
                    });
                }
            }
        } else {
            loaded_tables.push(load_table(cfg, rel, None)?);
        }
    }
    if loaded_tables.is_empty() {
        return Err(format!(
            "no readable table rows matched glob '{clean_glob}'"
        ));
    }

    let include_source_col = args_mentions_col(args, "_source_file");
    let include_sheet_col = args_mentions_col(args, "_sheet");

    let mut source_files: Vec<String> = Vec::with_capacity(loaded_tables.len());
    let mut unified_columns: Vec<String> = Vec::new();

    for tbl in &loaded_tables {
        let src_label = match &tbl.sheet {
            Some(sh) => format!("{}#{sh}", tbl.path),
            None => tbl.path.clone(),
        };
        if !source_files.contains(&src_label) {
            source_files.push(src_label);
        }
        for col in &tbl.columns {
            if !unified_columns
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(col))
            {
                unified_columns.push(col.clone());
            }
        }
    }
    if include_source_col
        && !unified_columns
            .iter()
            .any(|c| c.eq_ignore_ascii_case("_source_file"))
    {
        unified_columns.push("_source_file".to_string());
    }
    if include_sheet_col
        && !unified_columns
            .iter()
            .any(|c| c.eq_ignore_ascii_case("_sheet"))
    {
        unified_columns.push("_sheet".to_string());
    }

    let width = unified_columns.len();
    let mut unified_rows: Vec<Vec<String>> = Vec::new();
    for tbl in loaded_tables {
        let mut col_map: Vec<usize> = Vec::with_capacity(tbl.columns.len());
        for c in &tbl.columns {
            let idx = unified_columns
                .iter()
                .position(|u| u == c)
                .or_else(|| {
                    unified_columns
                        .iter()
                        .position(|u| u.eq_ignore_ascii_case(c))
                })
                .unwrap();
            col_map.push(idx);
        }
        let src_idx = unified_columns
            .iter()
            .position(|u| u.eq_ignore_ascii_case("_source_file"));
        let sheet_idx = unified_columns
            .iter()
            .position(|u| u.eq_ignore_ascii_case("_sheet"));
        for row in tbl.rows {
            if unified_rows.len() >= MAX_COMBINED_ROWS {
                return Err(format!(
                    "combined row count exceeds maximum of {MAX_COMBINED_ROWS}"
                ));
            }
            let mut out_row = vec![String::new(); width];
            for (ci, val) in row.into_iter().enumerate() {
                if let Some(&target_idx) = col_map.get(ci) {
                    out_row[target_idx] = val;
                }
            }
            if let Some(si) = src_idx {
                out_row[si] = tbl.path.clone();
            }
            if let Some(shi) = sheet_idx {
                out_row[shi] = tbl.sheet.clone().unwrap_or_default();
            }
            unified_rows.push(out_row);
        }
    }

    let query_res = run_table_pipeline(cfg, unified_columns, unified_rows, args, "aggregate_vault")?;
    Ok(VaultAggregateResult {
        columns: query_res.columns,
        rows: query_res.rows,
        source_files,
        total_rows: query_res.total_rows,
        matched_rows: query_res.matched_rows,
        saved_path: query_res.saved_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_parsing_and_formatting_are_exact() {
        assert_eq!(parse_decimal("$1,250.50"), Some(1_250_500_000_000));
        assert_eq!(parse_decimal("(42.00)"), Some(-42_000_000_000));
        assert_eq!(parse_decimal("0.1"), Some(100_000_000));
        assert_eq!(parse_decimal("0.2"), Some(200_000_000));
        assert_eq!(
            format_decimal(parse_decimal("0.1").unwrap() + parse_decimal("0.2").unwrap(), 9),
            "0.3"
        );
        assert_eq!(parse_decimal("02139"), None);
        assert_eq!(parse_decimal("2026-11-24"), None);
    }
}
