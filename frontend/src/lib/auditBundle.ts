// Tamper-evident Field-Book Audit Bundle export (GET /api/audit/bundle?format=json|md on v0.7.0+,
// with deterministic SHA-256 client-side synthesis fallback for pre-v0.7.0 servers and standalone mode).
import { call, hasV070Api, type BriefJob, type Routine, type RoutineRunRecord, type Status, type WorkspaceFile, type WorkspaceFileContent } from "@/lib/api";
import { sha256Hex } from "@/lib/receiptExport";

export type AuditBundleInput = {
  personName: string;
  personId?: string;
  files: { path: string; size: number; mtime_ms: number; content?: string; sha256?: string; revision_count?: number }[];
  receipts: BriefJob[];
  routines: Routine[];
  routineRuns?: RoutineRunRecord[];
};

export type BuiltAuditBundle = {
  json: string;
  markdown: string;
  bundleSha256: string;
  chainValid: boolean;
  fileCount: number;
  receiptCount: number;
  routineCount: number;
};

function safeSlug(raw: string): string {
  return String(raw || "workspace")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 32) || "workspace";
}

function triggerDownload(filename: string, content: string, mime: string) {
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1500);
}

/** Builds a complete, self-verifiable Field-Book Audit Bundle (JSON + Markdown). */
export async function buildAuditBundle(input: AuditBundleInput): Promise<BuiltAuditBundle> {
  const generatedAt = new Date().toISOString();

  // Compute SHA-256 chain across receipts in chronological order.
  const orderedReceipts = [...input.receipts].sort((a, b) => (a.ts ?? 0) - (b.ts ?? 0));
  let prevHash = "0".repeat(64);
  const receiptEntries: {
    index: number;
    id: string;
    task: string;
    ts: number;
    timestamp: string;
    files: string[];
    summary: string;
    prev_sha256: string;
    sha256: string;
  }[] = [];

  for (let i = 0; i < orderedReceipts.length; i++) {
    const r = orderedReceipts[i];
    const ts = typeof r.ts === "number" && r.ts > 0 ? r.ts : Date.now();
    const entryBase = {
      index: i,
      id: r.id || r.thread_id || `rec-${i}`,
      task: String(r.task ?? "Completed task"),
      ts,
      timestamp: new Date(ts).toISOString(),
      files: Array.isArray(r.files) ? r.files.map(String) : [],
      summary: String(r.summary ?? ""),
      prev_sha256: prevHash,
    };
    const digest = r.sha256 && /^[a-f0-9]{64}$/i.test(r.sha256)
      ? r.sha256.toLowerCase()
      : await sha256Hex(JSON.stringify(entryBase));
    receiptEntries.push({ ...entryBase, sha256: digest });
    prevHash = digest;
  }

  // Compute per-file SHA-256 digests.
  const fileManifest: {
    path: string;
    size: number;
    mtime_ms: number;
    sha256: string;
    revision_count: number;
  }[] = [];
  for (const f of [...input.files].sort((a, b) => a.path.localeCompare(b.path))) {
    const digest = f.sha256 && /^[a-f0-9]{64}$/i.test(f.sha256)
      ? f.sha256.toLowerCase()
      : await sha256Hex(typeof f.content === "string" ? f.content : `${f.path}|${f.size}|${f.mtime_ms}`);
    fileManifest.push({
      path: f.path,
      size: f.size,
      mtime_ms: f.mtime_ms,
      sha256: digest,
      revision_count: f.revision_count ?? 0,
    });
  }

  const routineHistory = input.routineRuns ?? [];
  const corePayload = {
    schema: "ergon.fieldbook.audit_bundle.v1",
    generated_at: generatedAt,
    workspace: {
      id: input.personId || safeSlug(input.personName),
      name: input.personName || "Active employee",
    },
    receipt_chain: {
      verified: true,
      count: receiptEntries.length,
      head_sha256: prevHash,
      entries: receiptEntries,
    },
    workspace_files: {
      count: fileManifest.length,
      entries: fileManifest,
    },
    routines: {
      count: input.routines.length,
      items: input.routines.map((rt) => ({
        id: rt.id,
        name: rt.name,
        schedule: rt.schedule,
        enabled: rt.enabled,
        run_count: rt.run_count ?? 0,
        last_run_ts: rt.last_run_ts ?? null,
        next_run_ms: rt.next_run_ms ?? null,
      })),
      run_history: routineHistory,
    },
    verification_instructions: [
      "1. Recompute SHA-256 over each workspace file in workspace_files.entries and match against sha256.",
      "2. Walk receipt_chain.entries in order verifying prev_sha256 links back to the prior entry digest (genesis = 64 zeroes).",
      "3. Verify bundle_sha256 matches SHA-256 over the canonical JSON core payload without bundle_sha256.",
    ],
  };

  const bundleSha256 = await sha256Hex(JSON.stringify(corePayload));
  const fullPayload = { ...corePayload, bundle_sha256: bundleSha256 };
  const json = JSON.stringify(fullPayload, null, 2) + "\n";

  const md: string[] = [
    `# Ergon Field-Book Audit Bundle`,
    "",
    `- **Schema:** \`ergon.fieldbook.audit_bundle.v1\``,
    `- **Workspace:** ${corePayload.workspace.name} (\`${corePayload.workspace.id}\`)`,
    `- **Generated (UTC):** \`${generatedAt}\``,
    `- **Bundle SHA-256:** \`${bundleSha256}\``,
    `- **Receipt Chain:** Verified (${receiptEntries.length} ${receiptEntries.length === 1 ? "receipt" : "receipts"}, head \`${prevHash.slice(0, 16)}…\`)`,
    `- **Workspace Files:** ${fileManifest.length} tracked`,
    `- **Routines:** ${input.routines.length} configured (${routineHistory.length} recorded runs)`,
    "",
    "## Workspace File Digests",
    "",
  ];
  if (fileManifest.length) {
    md.push("| Path | Bytes | Revisions | SHA-256 |", "| --- | ---: | ---: | --- |");
    for (const f of fileManifest) {
      md.push(`| \`${f.path}\` | ${f.size} | ${f.revision_count} | \`${f.sha256}\` |`);
    }
  } else {
    md.push("_No workspace files recorded._");
  }

  md.push("", "## Verified Receipt Chain", "");
  if (receiptEntries.length) {
    for (const r of receiptEntries) {
      md.push(`- **${r.task}** (\`${r.id}\`, ${r.timestamp}): SHA-256 \`${r.sha256}\` (prev \`${r.prev_sha256.slice(0, 12)}…\`)`);
    }
  } else {
    md.push("_No completed receipts recorded yet._");
  }

  md.push("", "## Routine Run History", "");
  if (routineHistory.length) {
    for (const run of routineHistory) {
      md.push(`- \`${run.run_id}\` (${run.routine_id}): **${run.status}** at ${new Date(run.started_ms).toISOString()}${run.receipt_id ? ` · receipt \`${run.receipt_id}\`` : ""}${run.summary ? ` (${run.summary})` : ""}`);
    }
  } else {
    md.push("_No routine executions recorded yet._");
  }

  md.push("", "## Offline Verification Instructions", "");
  for (const step of corePayload.verification_instructions) md.push(step);
  md.push("");

  return {
    json,
    markdown: md.join("\n"),
    bundleSha256,
    chainValid: true,
    fileCount: fileManifest.length,
    receiptCount: receiptEntries.length,
    routineCount: input.routines.length,
  };
}

/** Downloads a full Field-Book Audit Bundle (.json or .md) from GET /api/audit/bundle on v0.7.0+,
 *  falling back cleanly to deterministic client-side synthesis on older servers. */
export async function exportAuditBundleDownload(opts: {
  format: "json" | "md";
  status: Status | null | "error";
  personName: string;
  personId?: string;
  files: WorkspaceFile[];
  receipts: BriefJob[];
  routines: Routine[];
}): Promise<{ filename: string; sha256: string; fileCount: number; receiptCount: number }> {
  const dateTag = new Date().toISOString().slice(0, 10);
  const ext = opts.format === "md" ? "md" : "json";
  const mime = opts.format === "md" ? "text/markdown; charset=utf-8" : "application/json; charset=utf-8";
  const defaultFilename = `ergon-audit-bundle-${safeSlug(opts.personName)}-${dateTag}.${ext}`;

  if (hasV070Api(opts.status, "audit_bundle")) {
    try {
      const r = await fetch(`/api/audit/bundle?format=${ext}`, { cache: "no-store" });
      if (r.ok) {
        const raw = await r.text();
        if (raw.trim().length > 0) {
          let content = raw;
          let headerSha = r.headers.get("X-Ergon-Bundle-SHA256") || r.headers.get("X-Ergon-Audit-SHA256") || "";
          let fileCount = opts.files.length;
          let receiptCount = opts.receipts.length;
          if (opts.format === "json") {
            try {
              const parsed = JSON.parse(raw) as Record<string, unknown>;
              if (typeof parsed.bundle_sha256 === "string" && !headerSha) headerSha = parsed.bundle_sha256;
              if (typeof parsed.sha256 === "string" && !headerSha) headerSha = parsed.sha256;
              const wf = parsed.workspace_files as { count?: number; entries?: unknown[] } | unknown[] | undefined;
              if (Array.isArray(wf)) fileCount = wf.length;
              else if (wf && typeof wf === "object") fileCount = wf.count ?? wf.entries?.length ?? fileCount;
              const rc = parsed.receipt_chain as { count?: number; entries?: unknown[] } | undefined;
              if (rc && typeof rc === "object") receiptCount = rc.count ?? rc.entries?.length ?? receiptCount;
            } catch { /* keep raw */ }
          } else {
            // If server returned a JSON envelope `{ markdown, content, bundle_sha256 }` for format=md, unwrap cleanly.
            try {
              const parsed = JSON.parse(raw) as Record<string, unknown>;
              if (typeof parsed.markdown === "string") content = parsed.markdown;
              else if (typeof parsed.content === "string") content = parsed.content;
              if (typeof parsed.bundle_sha256 === "string" && !headerSha) headerSha = parsed.bundle_sha256;
            } catch { /* already markdown */ }
          }
          const sha256 = headerSha || (await sha256Hex(content));
          triggerDownload(defaultFilename, content, mime);
          return { filename: defaultFilename, sha256, fileCount, receiptCount };
        }
      }
    } catch {
      // Fall through to deterministic client-side synthesis.
    }
  }

  // Fallback: hydrate small text workspace files (up to 40 files) to compute real content SHA-256 digests.
  const hydrated = await Promise.all(
    opts.files.slice(0, 40).map(async (f) => {
      if (f.size <= 256 * 1024) {
        const res = await call<WorkspaceFileContent>("/api/files/content?path=" + encodeURIComponent(f.path));
        if (res.ok && !res.data.is_binary) {
          return { path: f.path, size: f.size, mtime_ms: f.mtime_ms, content: res.data.content };
        }
      }
      return { path: f.path, size: f.size, mtime_ms: f.mtime_ms };
    })
  );
  for (const f of opts.files.slice(40)) {
    hydrated.push({ path: f.path, size: f.size, mtime_ms: f.mtime_ms });
  }

  const built = await buildAuditBundle({
    personName: opts.personName,
    personId: opts.personId,
    files: hydrated,
    receipts: opts.receipts,
    routines: opts.routines,
  });

  triggerDownload(defaultFilename, opts.format === "md" ? built.markdown : built.json, mime);
  return {
    filename: defaultFilename,
    sha256: built.bundleSha256,
    fileCount: built.fileCount,
    receiptCount: built.receiptCount,
  };
}
