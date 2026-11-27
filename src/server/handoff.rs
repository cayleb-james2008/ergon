use super::*;

pub(super) struct PackCheckedFile {
    pub(super) path: String,
    pub(super) size_bytes: usize,
    pub(super) non_empty: bool,
    pub(super) count: usize,
    pub(super) unit: &'static str,
    pub(super) digest: String,
    pub(super) body: String,
}

pub(super) struct PackPreflight {
    pub(super) ready_count: usize,
    pub(super) empty_count: usize,
    pub(super) missing_paths: Vec<String>,
    pub(super) total_bytes: usize,
    pub(super) items: Vec<PackCheckedFile>,
    pub(super) citations: Vec<CitationCheck>,
    pub(super) citations_verified: usize,
    pub(super) citations_stale: usize,
    pub(super) citations_broken: usize,
}

impl PackPreflight {
    pub(super) fn freshness_pct(&self) -> u8 {
        (self.citations_verified * 100)
            .checked_div(self.citations.len())
            .map_or(100, |p| p as u8)
    }

    pub(super) fn citation_gate(&self) -> &'static str {
        if self.citations_broken == 0 && self.citations_stale == 0 {
            "100% Verified"
        } else if self.citations_stale > 0 {
            "Stale Line Drift"
        } else {
            "Broken Citations"
        }
    }

    pub(super) fn ok(&self) -> bool {
        self.ready_count > 0
            && self.empty_count == 0
            && self.missing_paths.is_empty()
            && self.citations_stale == 0
            && self.citations_broken == 0
    }

    pub(super) fn to_json(&self) -> Value {
        let mut files_json: Vec<Value> = self
            .items
            .iter()
            .map(|f| {
                json!({
                    "path": f.path,
                    "size_bytes": f.size_bytes,
                    "non_empty": f.non_empty,
                    "count": f.count,
                    "unit": f.unit,
                    "sha256": f.digest,
                    "status": if f.non_empty { "ready" } else { "empty" },
                })
            })
            .collect();
        for miss in &self.missing_paths {
            files_json.push(json!({
                "path": miss,
                "size_bytes": 0,
                "non_empty": false,
                "count": 0,
                "unit": "bytes",
                "sha256": "",
                "status": "missing",
            }));
        }
        json!({
            "ok": self.ok(),
            "ready_count": self.ready_count,
            "empty_count": self.empty_count,
            "missing_count": self.missing_paths.len(),
            "total_bytes": self.total_bytes,
            "citations_checked": self.citations.len(),
            "citations_verified": self.citations_verified,
            "citations_stale": self.citations_stale,
            "citations_broken": self.citations_broken,
            "citations_freshness_pct": self.freshness_pct(),
            "citation_gate": self.citation_gate(),
            "citations": self.citations.iter().map(CitationCheck::to_json).collect::<Vec<_>>(),
            "files": files_json,
        })
    }
}

pub(super) fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Find the pre-overwrite SHA-256 recorded just after `receipt_ts` (oldest snapshot with `ts > receipt_ts`).
pub(super) fn history_baseline_sha_for_receipt(
    data_dir: &std::path::Path,
    workspace_dir: &std::path::Path,
    rel_path: &str,
    receipt_ts: u64,
) -> Option<String> {
    if receipt_ts == 0 {
        return None;
    }
    let revs = file_history::list_revisions(data_dir, workspace_dir, rel_path).ok()?;
    revs.into_iter()
        .filter(|r| r.timestamp_ms > receipt_ts)
        .min_by_key(|r| r.timestamp_ms)
        .map(|r| r.sha256)
}

pub(super) fn status_severity(status: &str) -> u8 {
    match status {
        "verified" => 0,
        "stale" => 1,
        _ => 2,
    }
}

pub(super) fn adjust_citation_counts(status: &str, delta: isize, verified: &mut usize, stale: &mut usize, broken: &mut usize) {
    let target = match status {
        "verified" => verified,
        "stale" => stale,
        _ => broken,
    };
    if delta >= 0 {
        *target = target.saturating_add(delta as usize);
    } else {
        *target = target.saturating_sub((-delta) as usize);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_citations_into(
    data_dir: Option<&std::path::Path>,
    workspace_dir: &std::path::Path,
    text: &str,
    receipt_ts: u64,
    expected_shas: Option<&std::collections::HashMap<String, String>>,
    citations: &mut Vec<CitationCheck>,
    seen: &mut std::collections::HashMap<(String, usize, Option<usize>), usize>,
    verified: &mut usize,
    stale: &mut usize,
    broken: &mut usize,
) -> (usize, usize, usize, usize) {
    if text.is_empty() {
        return (0, 0, 0, 0);
    }
    let mut rep = verify_text_citations_against(workspace_dir, text, expected_shas);
    let mut run_total = 0usize;
    let mut run_verified = 0usize;
    let mut run_stale = 0usize;
    let mut run_broken = 0usize;
    for mut c in rep.citations.drain(..) {
        if c.status == "verified" && receipt_ts > 0 {
            if let Some(ddir) = data_dir {
                if let Some(base_sha) = history_baseline_sha_for_receipt(ddir, workspace_dir, &c.path, receipt_ts) {
                    if !base_sha.is_empty() && !base_sha.eq_ignore_ascii_case(&c.file_sha256) {
                        c.status = "stale";
                        c.baseline_sha256 = Some(base_sha);
                    }
                }
            }
        }
        run_total += 1;
        match c.status {
            "verified" => run_verified += 1,
            "stale" => run_stale += 1,
            _ => run_broken += 1,
        }
        let key = (c.path.clone(), c.line, c.end_line);
        if let Some(&existing_idx) = seen.get(&key) {
            let old_status = citations[existing_idx].status;
            if status_severity(c.status) > status_severity(old_status) {
                adjust_citation_counts(old_status, -1, verified, stale, broken);
                adjust_citation_counts(c.status, 1, verified, stale, broken);
                citations[existing_idx] = c;
            }
            continue;
        }
        if citations.len() >= 128 {
            continue;
        }
        let idx = citations.len();
        seen.insert(key, idx);
        adjust_citation_counts(c.status, 1, verified, stale, broken);
        citations.push(c);
    }
    (run_total, run_verified, run_stale, run_broken)
}

#[derive(Clone)]
pub(super) struct SummaryRunGate<'a> {
    pub(super) receipt: &'a receipts::ChainedReceipt,
    pub(super) checked: usize,
    pub(super) verified: usize,
    pub(super) stale: usize,
    pub(super) broken: usize,
}

impl SummaryRunGate<'_> {
    pub(super) fn gate_label(&self) -> &'static str {
        if self.checked == 0 {
            "Deliverables ready"
        } else if self.broken == 0 && self.stale == 0 {
            "Verified"
        } else if self.stale > 0 {
            "Stale Line Drift"
        } else {
            "Broken Citations"
        }
    }

    pub(super) fn to_json(&self) -> Value {
        json!({
            "id": self.receipt.id,
            "task": self.receipt.task,
            "ts": self.receipt.ts,
            "sha256": self.receipt.sha256,
            "files": self.receipt.files,
            "citations_checked": self.checked,
            "citations_verified": self.verified,
            "citations_stale": self.stale,
            "citations_broken": self.broken,
            "citation_gate": self.gate_label(),
        })
    }
}

#[inline(never)]
pub(super) fn build_summary_run_gates<'a>(
    data_dir: &std::path::Path,
    workspace_dir: &std::path::Path,
    summary_runs: &[&'a receipts::ChainedReceipt],
) -> Vec<SummaryRunGate<'a>> {
    let mut gates = Vec::with_capacity(summary_runs.len());
    if summary_runs.is_empty() {
        return gates;
    }
    let mut dummy_cits = Vec::new();
    let mut dummy_seen = std::collections::HashMap::new();
    for &rcpt in summary_runs {
        dummy_cits.clear();
        dummy_seen.clear();
        let mut v = 0usize;
        let mut s = 0usize;
        let mut b = 0usize;
        let (tot, rv, rs, rb) = collect_citations_into(
            Some(data_dir),
            workspace_dir,
            &rcpt.summary,
            rcpt.ts,
            None,
            &mut dummy_cits,
            &mut dummy_seen,
            &mut v,
            &mut s,
            &mut b,
        );
        gates.push(SummaryRunGate {
            receipt: rcpt,
            checked: tot,
            verified: rv,
            stale: rs,
            broken: rb,
        });
    }
    gates
}

pub(super) fn receipt_matches_handoff_scope(
    rcpt: &receipts::ChainedReceipt,
    scope: &audit_bundle::HandoffScope,
    resolver: &search::ScopeResolver,
) -> bool {
    let r_eid = rcpt
        .engagement_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if let Some(ref target_eid) = scope.engagement_id {
        return r_eid == Some(target_eid.as_str());
    }
    if let Some(ref target_cid) = scope.client_id {
        return r_eid.and_then(|eid| {
            resolver
                .store
                .engagements
                .iter()
                .find(|e| e.id == eid)
                .map(|e| e.client_id.as_str())
        }) == Some(target_cid.as_str());
    }
    true
}

#[cold]
#[inline(never)]
pub(super) fn build_handoff_summary_with_idx(
    cfg: &Config,
    person: &Person,
    title: &str,
    idx: Option<&search::PersonIndex>,
) -> String {
    build_handoff_summary_scoped(cfg, person, title, idx, None)
}

#[cold]
#[inline(never)]
pub(super) fn build_handoff_summary_scoped(
    cfg: &Config,
    person: &Person,
    title: &str,
    idx: Option<&search::PersonIndex>,
    scope: Option<&audit_bundle::HandoffScope>,
) -> String {
    let mut requested = Vec::new();
    let mut summary_runs: Vec<&receipts::ChainedReceipt> = Vec::new();
    let mut extra_receipts: Vec<(&str, u64)> = Vec::new();
    let resolver = scope.map(|_| {
        let root_data_dir = records::infer_root_data_dir(&cfg.data_dir);
        search::ScopeResolver::load(&root_data_dir, &cfg.workspace_dir)
    });
    if let Some(idx) = idx {
        let (_, items) = idx.list_files_scoped(
            0,
            Some(15),
            "",
            scope.and_then(|s| s.client_id.as_deref()),
            scope.and_then(|s| s.engagement_id.as_deref()),
        );
        for item in items {
            if let Some(p) = item["path"].as_str() {
                requested.push(p.to_string());
            }
        }
        for rcpt in idx.chain.iter().rev() {
            if let (Some(sc), Some(res)) = (scope, resolver.as_ref()) {
                if !receipt_matches_handoff_scope(rcpt, sc, res) {
                    continue;
                }
            }
            if rcpt.finished && !rcpt.answered && summary_runs.len() < 15 {
                summary_runs.push(rcpt);
                for f in &rcpt.files {
                    if let (Some(sc), Some(res)) = (scope, resolver.as_ref()) {
                        if !res.matches_path(f, sc.client_id.as_deref(), sc.engagement_id.as_deref()) {
                            continue;
                        }
                    }
                    if requested.len() < 25 && !requested.iter().any(|x| x == f) {
                        requested.push(f.clone());
                    }
                }
            }
            if extra_receipts.len() < 15 && !rcpt.summary.is_empty() {
                extra_receipts.push((rcpt.summary.as_str(), rcpt.ts));
            }
            if summary_runs.len() >= 15 && extra_receipts.len() >= 15 {
                break;
            }
        }
    }
    if let Ok(preflight) = inspect_workspace_pack(
        &cfg.data_dir,
        &cfg.workspace_dir,
        &requested,
        false,
        &extra_receipts,
        None,
    ) {
        let summary_gates = build_summary_run_gates(&cfg.data_dir, &cfg.workspace_dir, &summary_runs);
        return format_workspace_pack_markdown(&preflight, &person.name, title, &summary_gates);
    }
    let bundle = audit_bundle::build_audit_bundle(cfg, person);
    audit_bundle::render_audit_bundle_markdown(&bundle)
}

#[cold]
#[inline(never)]
pub fn build_handoff_summary_markdown(cfg: &Config, person: &Person, title: &str) -> String {
    let idx = search::SearchCache::new().get(
        &person.id,
        &cfg.data_dir,
        &cfg.workspace_dir,
        &cfg.memory_path,
    );
    build_handoff_summary_with_idx(cfg, person, title, Some(&idx))
}

#[cold]
#[inline(never)]
pub(super) fn handle_export_handoff(
    stream: &mut ConnStream,
    state: &AppState,
    req: &Request,
    person: &Person,
    runtime: &Arc<PersonRuntime>,
) -> std::io::Result<()> {
    let parsed: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
    let dry_run = parsed["dry_run"].as_bool().unwrap_or_else(|| {
        query_param(&req.query, "dry_run")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    });
    let force = parsed["force"].as_bool().unwrap_or_else(|| {
        query_param(&req.query, "force")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    });
    let title = parsed["title"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Client Handoff");
    let req_client_id = parsed["client_id"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            query_param(&req.query, "client_id")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        });
    let req_engagement_id = parsed["engagement_id"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            query_param(&req.query, "engagement_id")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        });

    let policy = crate::load_workspace_policy(&runtime.cfg.data_dir);
    let scope = audit_bundle::HandoffScope::resolve(
        &state.cfg.data_dir,
        &person.id,
        req_client_id.as_deref(),
        req_engagement_id.as_deref(),
        &policy,
    );

    let idx = person_idx(state, person, runtime);
    let summary_md =
        build_handoff_summary_scoped(&runtime.cfg, person, title, Some(&idx), scope.as_ref());
    let mut eff_cfg = runtime.cfg.clone();
    eff_cfg.api_key = effective_cfg(state, runtime).api_key;

    if dry_run {
        let scan = audit_bundle::scan_handoff_leaks_scoped(
            &eff_cfg,
            &[("CLIENT_SUMMARY.md", &summary_md)],
            eff_cfg.api_key.as_deref(),
            scope.as_ref(),
        );
        let strict_block = policy.profile == "strict_client"
            || policy.strict_citation_gate
            || (policy.isolate_active_engagement && scan.cross_client_count > 0);
        let would_block = strict_block && !scan.clean;
        return respond_json(
            stream,
            200,
            &json!({
                "ok": !would_block,
                "dry_run": true,
                "would_block": would_block,
                "scope": {
                    "client_id": scope.as_ref().and_then(|s| s.client_id.clone()),
                    "engagement_id": scope.as_ref().and_then(|s| s.engagement_id.clone()),
                },
                "policy": policy.to_json(&person.id),
                "scan": scan.to_json(),
            }),
        );
    }

    let security = tripwire::audit_section(&state.cfg.data_dir, &person.id);
    let cached = (idx.chain.as_slice(), idx.chain_valid, idx.head_hash.as_str());
    match audit_bundle::build_handoff_bundle_zip_scoped(
        &eff_cfg,
        person,
        title,
        &summary_md,
        &security,
        &policy,
        force,
        Some(cached),
        false,
        req_client_id.as_deref(),
        req_engagement_id.as_deref(),
    ) {
        Err(blocked) => respond_json(stream, 409, &blocked.to_json()),
        Ok(res) => {
            let headers = vec![
                ("X-Content-Type-Options", "nosniff".to_string()),
                (
                    "Content-Disposition",
                    "attachment; filename=\"ergon-client-handoff.zip\"".to_string(),
                ),
                ("X-Ergon-Handoff-SHA256", res.sha256),
                ("X-Ergon-Handoff-Leaks", res.scan.leak_count.to_string()),
                ("X-Ergon-Workspace-Policy", policy.profile.to_string()),
            ];
            respond_bytes(stream, 200, "application/zip", &res.zip_bytes, &headers)
        }
    }
}

/// Validates workspace deliverable paths, computes non-empty/row/line counts, citations, and SHA-256 digests.
pub(super) fn inspect_workspace_pack(
    data_dir: &std::path::Path,
    workspace_dir: &std::path::Path,
    requested: &[String],
    explicit: bool,
    extra_receipt_texts: &[(&str, u64)],
    expected_shas: Option<&std::collections::HashMap<String, String>>,
) -> Result<PackPreflight, (u16, String)> {
    const MAX_PACK_BYTES: usize = 2 * 1024 * 1024;
    const MAX_FILE_BYTES: usize = 512 * 1024;
    let mut items: Vec<PackCheckedFile> = Vec::new();
    let mut missing_paths: Vec<String> = Vec::new();
    let mut ready_count = 0usize;
    let mut empty_count = 0usize;
    let mut total_bytes = 0usize;
    let mut rendered_bytes = 0usize;
    let mut citations: Vec<CitationCheck> = Vec::new();
    let mut seen_citations: std::collections::HashMap<(String, usize, Option<usize>), usize> =
        std::collections::HashMap::new();
    let mut citations_verified = 0usize;
    let mut citations_stale = 0usize;
    let mut citations_broken = 0usize;
    for &(rtext, rts) in extra_receipt_texts {
        collect_citations_into(
            Some(data_dir),
            workspace_dir,
            rtext,
            rts,
            expected_shas,
            &mut citations,
            &mut seen_citations,
            &mut citations_verified,
            &mut citations_stale,
            &mut citations_broken,
        );
    }
    for rel in requested {
        let full = match sandbox_path(workspace_dir, rel) {
            Ok(p) => p,
            Err(e) => {
                if explicit {
                    return Err((403, e));
                }
                continue;
            }
        };
        if full.symlink_metadata().is_ok_and(|md| md.file_type().is_symlink()) {
            if explicit {
                return Err((403, "cannot pack symlinked path".to_string()));
            }
            continue;
        }
        let Ok(bytes) = std::fs::read(&full) else {
            if explicit {
                missing_paths.push(rel.clone());
            }
            continue;
        };
        let digest = receipts::sha256_hex(&bytes);
        let ext = full
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let non_empty = !bytes.iter().all(|b| b.is_ascii_whitespace());
        if non_empty {
            ready_count += 1;
        } else {
            empty_count += 1;
        }
        total_bytes += bytes.len();
        let (count, unit, body) = if bytes.len() > MAX_FILE_BYTES {
            (
                bytes.len(),
                "bytes",
                format!(
                    "_Skipped inline preview ({} bytes exceeds 512 KB cap; SHA-256 `{}`)._",
                    bytes.len(),
                    digest
                ),
            )
        } else if let Ok(text) = std::str::from_utf8(&bytes) {
            let line_count = text.lines().filter(|l| !l.trim().is_empty()).count();
            let (metric, unit_name) = if ext == "csv" || ext == "tsv" {
                (line_count.saturating_sub(1), "rows")
            } else {
                (line_count, "lines")
            };
            collect_citations_into(
                Some(data_dir),
                workspace_dir,
                text,
                0,
                expected_shas,
                &mut citations,
                &mut seen_citations,
                &mut citations_verified,
                &mut citations_stale,
                &mut citations_broken,
            );
            let rendered = if ext == "md" || ext == "markdown" || ext == "txt" {
                text.trim_end().to_string()
            } else {
                let fence = match ext.as_str() {
                    "csv" | "tsv" => "csv",
                    "json" => "json",
                    _ => "text",
                };
                format!("```{fence}\n{}\n```", text.trim_end())
            };
            (metric, unit_name, rendered)
        } else {
            (
                bytes.len(),
                "bytes",
                format!(
                    "_Binary deliverable ({} bytes, `.{ext}`; download original file from workspace; SHA-256 `{}`)._",
                    bytes.len(),
                    digest
                ),
            )
        };
        if rendered_bytes + body.len() > MAX_PACK_BYTES {
            break;
        }
        rendered_bytes += body.len();
        items.push(PackCheckedFile {
            path: rel.clone(),
            size_bytes: bytes.len(),
            non_empty,
            count,
            unit,
            digest,
            body,
        });
    }
    if explicit && items.is_empty() {
        return Err((404, "no matching workspace files found".to_string()));
    }
    Ok(PackPreflight {
        ready_count,
        empty_count,
        missing_paths,
        total_bytes,
        items,
        citations,
        citations_verified,
        citations_stale,
        citations_broken,
    })
}

#[inline(never)]
pub(super) fn format_workspace_pack_markdown(
    preflight: &PackPreflight,
    person_name: &str,
    title: &str,
    summary_runs: &[SummaryRunGate<'_>],
) -> String {
    let ok = preflight.ok();
    let ready = preflight.ready_count;
    let status_word = if ok { "PASS" } else { "WARN" };
    let total_cits = preflight.citations.len();
    let ratio_label = if total_cits == 0 {
        "no inline citations".to_string()
    } else {
        format!(
            "{}/{} verified ({}%)",
            preflight.citations_verified,
            total_cits,
            preflight.freshness_pct()
        )
    };
    let mut md = format!(
        "# {title}\n\n- Prepared for: **{person_name}**\n- Deliverables included: {}\n- Completed runs summarized: {}\n- Pre-flight check: **{status_word}** ({ready} non-empty, {} empty, {} missing, {} total bytes; citations: {} — {ratio_label})\n\n",
        preflight.items.len(),
        summary_runs.len(),
        preflight.empty_count,
        preflight.missing_paths.len(),
        preflight.total_bytes,
        preflight.citation_gate(),
    );
    if !summary_runs.is_empty() {
        md.push_str("## Verified Work Summary\n\n");
        if preflight.citations_stale > 0 || preflight.citations_broken > 0 {
            md.push_str(&format!(
                "- **Batch Citation Gate:** WARN — {}/{} verified, **{} stale**, {} broken across {} runs\n\n",
                preflight.citations_verified,
                total_cits,
                preflight.citations_stale,
                preflight.citations_broken,
                summary_runs.len(),
            ));
        } else if total_cits > 0 {
            md.push_str(&format!(
                "- **Batch Citation Gate:** 100% Verified ({}/{} citations across {} runs)\n\n",
                preflight.citations_verified,
                total_cits,
                summary_runs.len(),
            ));
        }
        for sg in summary_runs {
            let r = sg.receipt;
            let short_sha = &r.sha256[..12.min(r.sha256.len())];
            let snippet = truncate(r.summary.trim(), 180);
            let files_part = if r.files.is_empty() {
                String::new()
            } else {
                format!(" · Files: {}", r.files.join(", "))
            };
            let cit_part = if sg.checked == 0 {
                String::new()
            } else if sg.stale > 0 || sg.broken > 0 {
                format!(
                    " · **[CITATION ALERT: {}/{} verified, {} stale, {} broken]**",
                    sg.verified, sg.checked, sg.stale, sg.broken
                )
            } else {
                format!(" · [citations: {}/{} verified]", sg.verified, sg.checked)
            };
            md.push_str(&format!(
                "- **{}** (`{}` · `sha256:{}`{files_part}{cit_part}): {}\n",
                r.task, r.id, short_sha, snippet
            ));
        }
        md.push('\n');
    }
    md.push_str("## Included Deliverables\n\n");
    for f in &preflight.items {
        md.push_str(&format!(
            "- `{}` ({} {}, `sha256:{}`)\n",
            f.path, f.count, f.unit, f.digest
        ));
    }
    if !preflight.citations.is_empty() {
        md.push_str("\n## Citation Verification\n\n");
        for c in &preflight.citations {
            md.push_str(&format!(
                "- `{}` -> **{}** ({} lines in file, live `sha256:{}`)\n",
                c.raw,
                c.status,
                c.total_lines,
                &c.file_sha256[..12.min(c.file_sha256.len())]
            ));
        }
    }
    for f in &preflight.items {
        md.push_str(&format!(
            "\n---\n\n## `{}`\n\n_SHA-256: `{}` · {} {} · {} bytes_\n\n{}\n",
            f.path, f.digest, f.count, f.unit, f.size_bytes, f.body
        ));
    }
    md
}

/// Builds a Markdown Client Deliverable Pack across workspace files with pre-flight verification & SHA-256 digests.
pub(super) fn build_workspace_pack(
    workspace_dir: &std::path::Path,
    person_name: &str,
    title: &str,
    requested: &[String],
    explicit: bool,
) -> Result<(String, String, bool, usize), (u16, String)> {
    let preflight = inspect_workspace_pack(workspace_dir, workspace_dir, requested, explicit, &[], None)?;
    let ok = preflight.ok();
    let ready = preflight.ready_count;
    let md = format_workspace_pack_markdown(&preflight, person_name, title, &[]);
    let pack_sha = receipts::sha256_hex(md.as_bytes());
    Ok((md, pack_sha, ok, ready))
}

/// Builds a standalone, zero-external-request, printable Client Handoff (`.html`) with SHA-256 Manifest.
#[inline(never)]
pub(super) fn build_workspace_handoff_html(
    preflight: &PackPreflight,
    person_name: &str,
    title: &str,
    delta_md: Option<&str>,
    summary_runs: &[SummaryRunGate<'_>],
) -> (String, String) {
    let gate = preflight.citation_gate();
    let gate_class = if preflight.ok() { "ok" } else { "warn" };
    let total_cits = preflight.citations.len();
    let cit_ratio = if total_cits == 0 {
        "0 inline citations".to_string()
    } else {
        format!(
            "{}/{} verified ({}%)",
            preflight.citations_verified,
            total_cits,
            preflight.freshness_pct()
        )
    };
    let mut summary_section = String::new();
    if !summary_runs.is_empty() {
        summary_section.push_str("<section><h2>Verified Work Summary</h2><table><thead><tr><th>Task</th><th>Summary</th><th>Deliverables</th><th>Citations</th><th>Receipt SHA-256</th></tr></thead><tbody>");
        for sg in summary_runs {
            let r = sg.receipt;
            let short_sha = &r.sha256[..12.min(r.sha256.len())];
            let cit_pill_cls = if sg.stale > 0 || sg.broken > 0 { "warn" } else { "ok" };
            let cit_pill_txt = if sg.checked == 0 {
                "ready".to_string()
            } else if sg.stale > 0 || sg.broken > 0 {
                format!("{}/{} ({} stale)", sg.verified, sg.checked, sg.stale + sg.broken)
            } else {
                format!("{}/{} verified", sg.verified, sg.checked)
            };
            summary_section.push_str(&format!(
                "<tr><td><strong>{}</strong></td><td>{}</td><td><code>{}</code></td><td><span class=\"pill {}\">{}</span></td><td><code>{}</code></td></tr>",
                html_escape(&r.task),
                html_escape(&truncate(r.summary.trim(), 180)),
                html_escape(&r.files.join(", ")),
                cit_pill_cls,
                html_escape(&cit_pill_txt),
                html_escape(short_sha)
            ));
        }
        summary_section.push_str("</tbody></table></section>");
    }
    let mut files_rows = String::new();
    for f in &preflight.items {
        files_rows.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{} {}</td><td>{} B</td><td><code>{}</code></td></tr>\n",
            html_escape(&f.path),
            f.count,
            f.unit,
            f.size_bytes,
            html_escape(&f.digest)
        ));
    }
    let mut cit_section = String::new();
    if !preflight.citations.is_empty() {
        cit_section.push_str("<section><h2>Live Citation Verification</h2><table><thead><tr><th>Citation</th><th>Gate Status</th><th>Live File SHA-256</th><th>Excerpt</th></tr></thead><tbody>");
        for c in &preflight.citations {
            let short_sha = if c.file_sha256.is_empty() {
                "—"
            } else {
                &c.file_sha256[..12.min(c.file_sha256.len())]
            };
            cit_section.push_str(&format!(
                "<tr><td><code>{}</code></td><td><span class=\"pill {}\">{}</span></td><td><code>{}</code></td><td>{}</td></tr>",
                html_escape(&c.raw),
                if c.status == "verified" { "ok" } else { "warn" },
                html_escape(c.status),
                html_escape(short_sha),
                html_escape(&c.line_excerpt)
            ));
        }
        cit_section.push_str("</tbody></table></section>");
    }
    let delta_section = delta_md
        .filter(|d| !d.trim().is_empty())
        .map(|d| format!("<section><h2>Verified Delta</h2><pre>{}</pre></section>", html_escape(d)))
        .unwrap_or_default();
    let mut previews = String::new();
    for f in &preflight.items {
        previews.push_str(&format!(
            "<section class=\"doc\"><h3><code>{}</code></h3><p class=\"meta\">SHA-256: <code>{}</code> &middot; {} {} &middot; {} bytes</p><pre>{}</pre></section>\n",
            html_escape(&f.path),
            html_escape(&f.digest),
            f.count,
            f.unit,
            f.size_bytes,
            html_escape(&f.body)
        ));
    }
    let manifest_seed = preflight
        .items
        .iter()
        .map(|f| format!("{}:{}", f.path, f.digest))
        .collect::<Vec<_>>()
        .join("|");
    let manifest_sha = receipts::sha256_hex(manifest_seed.as_bytes());
    let html = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><style>:root{{color-scheme:light}}body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;max-width:920px;margin:32px auto;padding:0 20px;color:#111827;background:#fff;line-height:1.55}}header{{border-bottom:1px solid #e5e7eb;padding-bottom:16px;margin-bottom:24px}}h1{{margin:0 0 8px;font-size:24px}}h2{{font-size:17px;margin:28px 0 10px}}h3{{font-size:14px;margin:0 0 6px}}.meta{{color:#4b5563;font-size:13px;margin:4px 0 12px}}.pill{{display:inline-block;padding:2px 9px;border-radius:999px;font-size:12px;font-weight:600}}.pill.ok{{background:#dcfce7;color:#166534}}.pill.warn{{background:#fef3c7;color:#92400e}}table{{width:100%;border-collapse:collapse;font-size:13px}}th,td{{text-align:left;padding:8px 10px;border-bottom:1px solid #e5e7eb;vertical-align:top}}th{{background:#f9fafb;font-weight:600}}code{{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:12px;word-break:break-all}}pre{{background:#f9fafb;border:1px solid #e5e7eb;border-radius:8px;padding:12px;overflow:auto;font-size:12px;white-space:pre-wrap;word-break:break-word}}.doc{{margin-top:20px;break-inside:avoid}}@media print{{body{{margin:0}}header,section{{break-inside:avoid}}}}</style></head><body><header><h1>{}</h1><p class=\"meta\">Prepared for <strong>{}</strong> &middot; <span class=\"pill {}\">{}</span> &middot; Citations: {} &middot; Manifest SHA-256: <code>{}</code></p></header>{}<section><h2>SHA-256 Deliverable Manifest</h2><table><thead><tr><th>File</th><th>Contents</th><th>Size</th><th>SHA-256</th></tr></thead><tbody>{}</tbody></table></section>{}{}<section><h2>Deliverable Previews</h2>{}</section></body></html>",
        html_escape(title),
        html_escape(title),
        html_escape(person_name),
        gate_class,
        html_escape(gate),
        html_escape(&cit_ratio),
        html_escape(&manifest_sha),
        summary_section,
        files_rows,
        cit_section,
        delta_section,
        previews
    );
    let pack_sha = receipts::sha256_hex(html.as_bytes());
    (html, pack_sha)
}

pub(super) fn build_receipt_compare(a: &receipts::ChainedReceipt, b: &receipts::ChainedReceipt) -> (Value, String) {
    let diff = file_history::diff_texts(
        &format!("{} ({})", a.id, a.task),
        &format!("{} ({})", b.id, b.task),
        &a.summary,
        &b.summary,
    );
    let files_added: Vec<String> = b
        .files
        .iter()
        .filter(|f| !a.files.contains(f))
        .cloned()
        .collect();
    let files_removed: Vec<String> = a
        .files
        .iter()
        .filter(|f| !b.files.contains(f))
        .cloned()
        .collect();
    let files_shared: Vec<String> = b
        .files
        .iter()
        .filter(|f| a.files.contains(f))
        .cloned()
        .collect();
    let line_delta = diff.added as isize - diff.removed as isize;
    let file_delta = b.files.len() as isize - a.files.len() as isize;
    let mut delta_md = format!(
        "## Verified Run Delta (`{}` -> `{}`)\n\n- **Baseline:** `{}` — {} (`sha256:{}`)\n- **Target:** `{}` — {} (`sha256:{}`)\n- **Summary Diff:** +{} / -{} lines ({}% similarity, net {:+} lines)\n- **File Outputs:** {} added, {} removed, {} shared (net {:+} files)\n",
        a.id,
        b.id,
        a.id,
        a.task,
        &a.sha256[..12.min(a.sha256.len())],
        b.id,
        b.task,
        &b.sha256[..12.min(b.sha256.len())],
        diff.added,
        diff.removed,
        diff.similarity_pct,
        line_delta,
        files_added.len(),
        files_removed.len(),
        files_shared.len(),
        file_delta
    );
    if !files_added.is_empty() {
        delta_md.push_str(&format!("- **New Deliverables:** {}\n", files_added.join(", ")));
    }
    if !files_removed.is_empty() {
        delta_md.push_str(&format!("- **Removed Deliverables:** {}\n", files_removed.join(", ")));
    }
    if !diff.unified.trim().is_empty() {
        delta_md.push_str(&format!("\n```diff\n{}\n```\n", diff.unified.trim_end()));
    }
    let payload = json!({
        "receipt_a": a.to_json(),
        "receipt_b": b.to_json(),
        "added": diff.added,
        "removed": diff.removed,
        "unchanged": diff.unchanged,
        "similarity_pct": diff.similarity_pct,
        "line_delta": line_delta,
        "file_delta": file_delta,
        "files_added": files_added,
        "files_removed": files_removed,
        "files_shared": files_shared,
        "hunks": diff.hunks.iter().map(file_history::DiffHunk::to_json).collect::<Vec<_>>(),
        "unified": diff.unified,
        "delta_markdown": delta_md,
    });
    (payload, delta_md)
}

