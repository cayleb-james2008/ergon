// Real SHA-256 Receipt Export (.md / .json) using Web Crypto SubtleCrypto with a deterministic
// pure-JS SHA-256 fallback so cryptographic digests work identically over HTTPS, loopback HTTP,
// and offline standalone pages, plus seamless server /api/receipts/:id/export integration.

export type ExportableReceipt = {
  id: string;
  task: string;
  person?: string;
  status?: string;
  summary?: string;
  ts?: number;
  files?: string[];
  steps?: string[];
  filePath?: string;
  fileContent?: string;
  prev_sha256?: string;
};

// Standard FIPS 180-4 SHA-256 fallback for environments where crypto.subtle is unavailable.
const K = new Uint32Array([
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
]);

function sha256Sync(text: string): string {
  const bytes = new TextEncoder().encode(text);
  const bitLen = bytes.length * 8;
  const padLen = ((bytes.length + 9 + 63) >> 6) << 6;
  const buf = new Uint8Array(padLen);
  buf.set(bytes);
  buf[bytes.length] = 0x80;
  const view = new DataView(buf.buffer);
  view.setUint32(padLen - 8, Math.floor(bitLen / 0x100000000), false);
  view.setUint32(padLen - 4, bitLen >>> 0, false);

  let h0 = 0x6a09e667;
  let h1 = 0xbb67ae85;
  let h2 = 0x3c6ef372;
  let h3 = 0xa54ff53a;
  let h4 = 0x510e527f;
  let h5 = 0x9b05688c;
  let h6 = 0x1f83d9ab;
  let h7 = 0x5be0cd19;
  const w = new Uint32Array(64);
  const rotr = (x: number, n: number) => (x >>> n) | (x << (32 - n));

  for (let off = 0; off < padLen; off += 64) {
    for (let i = 0; i < 16; i++) w[i] = view.getUint32(off + i * 4, false);
    for (let i = 16; i < 64; i++) {
      const s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >>> 3);
      const s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >>> 10);
      w[i] = (w[i - 16] + s0 + w[i - 7] + s1) >>> 0;
    }
    let a = h0, b = h1, c = h2, d = h3, e = h4, f = h5, g = h6, h = h7;
    for (let i = 0; i < 64; i++) {
      const S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
      const ch = (e & f) ^ (~e & g);
      const t1 = (h + S1 + ch + K[i] + w[i]) >>> 0;
      const S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
      const maj = (a & b) ^ (a & c) ^ (b & c);
      const t2 = (S0 + maj) >>> 0;
      h = g; g = f; f = e; e = (d + t1) >>> 0;
      d = c; c = b; b = a; a = (t1 + t2) >>> 0;
    }
    h0 = (h0 + a) >>> 0; h1 = (h1 + b) >>> 0; h2 = (h2 + c) >>> 0; h3 = (h3 + d) >>> 0;
    h4 = (h4 + e) >>> 0; h5 = (h5 + f) >>> 0; h6 = (h6 + g) >>> 0; h7 = (h7 + h) >>> 0;
  }

  return [h0, h1, h2, h3, h4, h5, h6, h7]
    .map((n) => n.toString(16).padStart(8, "0"))
    .join("");
}

export async function sha256Hex(text: string): Promise<string> {
  try {
    if (typeof crypto !== "undefined" && crypto.subtle) {
      const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
      return Array.from(new Uint8Array(digest))
        .map((b) => b.toString(16).padStart(2, "0"))
        .join("");
    }
  } catch {
    // Fall back to deterministic FIPS 180-4 implementation.
  }
  return sha256Sync(text);
}

function safeSlug(raw: string): string {
  return String(raw || "receipt")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 40) || "receipt";
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

export async function buildSignedReceipt(input: ExportableReceipt): Promise<{
  payload: Record<string, unknown>;
  sha256: string;
  markdown: string;
  json: string;
}> {
  const ts = typeof input.ts === "number" && input.ts > 0 ? input.ts : Date.now();
  const iso = new Date(ts).toISOString();
  const fileHash = typeof input.fileContent === "string" ? await sha256Hex(input.fileContent) : undefined;
  const canonical = {
    schema: "ergon.fieldbook.receipt.v1",
    id: input.id,
    task: input.task,
    person: input.person || "Active employee",
    status: input.status || "finished",
    timestamp: iso,
    summary: input.summary || "",
    files: Array.isArray(input.files) ? input.files : input.filePath ? [input.filePath] : [],
    steps: Array.isArray(input.steps) ? input.steps : [],
    file_path: input.filePath || undefined,
    file_sha256: fileHash,
    prev_sha256: input.prev_sha256 || "0".repeat(64),
  };
  const sha256 = await sha256Hex(JSON.stringify(canonical));
  const payload = { ...canonical, sha256 };

  const mdLines = [
    `# Ergon Field-Book Receipt`,
    "",
    `- **Receipt ID:** \`${canonical.id}\``,
    `- **Task / Subject:** ${canonical.task}`,
    `- **Employee Desk:** ${canonical.person}`,
    `- **Status:** ${canonical.status}`,
    `- **Timestamp (UTC):** \`${iso}\``,
    `- **SHA-256 Digest:** \`${sha256}\``,
    `- **Previous Chain Hash:** \`${canonical.prev_sha256}\``,
  ];
  if (fileHash && canonical.file_path) {
    mdLines.push(`- **File Verified:** \`${canonical.file_path}\` (SHA-256 \`${fileHash}\`)`);
  }
  if (canonical.summary) {
    mdLines.push("", "## Outcome Summary", "", canonical.summary);
  }
  if (canonical.files.length) {
    mdLines.push("", "## Workspace Files", "");
    for (const f of canonical.files) mdLines.push(`- \`${f}\``);
  }
  if (canonical.steps.length) {
    mdLines.push("", "## Verified Tool Ledger", "");
    canonical.steps.forEach((s, idx) => {
      const firstLine = String(s).split("\n")[0];
      mdLines.push(`${idx + 1}. \`${firstLine}\``);
    });
  }

  return {
    payload,
    sha256,
    markdown: mdLines.join("\n") + "\n",
    json: JSON.stringify(payload, null, 2) + "\n",
  };
}

/** Export a Field-Book receipt via server /api/receipts/:id/export when enabled, or verified client-side SHA-256. */
export async function exportReceiptDownload(
  input: ExportableReceipt,
  format: "md" | "json",
  useServerEndpoint: boolean
): Promise<{ filename: string; sha256: string }> {
  const slug = safeSlug(input.filePath || input.task || input.id);
  const filename = `ergon-receipt-${slug}.${format}`;

  if (useServerEndpoint && input.id && !input.fileContent) {
    try {
      const url = `/api/receipts/${encodeURIComponent(input.id)}/export?format=${format}`;
      const r = await fetch(url, { cache: "no-store" });
      if (r.ok) {
        const text = await r.text();
        if (text.trim().length > 0) {
          const headerHash = r.headers.get("X-Ergon-Receipt-SHA256") || "";
          const matchHash = /[a-f0-9]{64}/i.exec(text)?.[0] || "";
          const digest = headerHash || matchHash || (await sha256Hex(text));
          triggerDownload(
            filename,
            text,
            format === "json" ? "application/json; charset=utf-8" : "text/markdown; charset=utf-8"
          );
          return { filename, sha256: digest };
        }
      }
    } catch {
      // Fall through cleanly to client-side cryptographic export.
    }
  }

  const built = await buildSignedReceipt(input);
  triggerDownload(
    filename,
    format === "json" ? built.json : built.markdown,
    format === "json" ? "application/json; charset=utf-8" : "text/markdown; charset=utf-8"
  );
  return { filename, sha256: built.sha256 };
}

export type PackPreflightFile = {
  path: string;
  size_bytes: number;
  non_empty: boolean;
  count: number;
  unit: "rows" | "lines" | "bytes";
  sha256: string;
  status: "ready" | "empty" | "missing";
};

export type CitationCheck = {
  raw: string;
  path: string;
  line: number;
  end_line?: number | null;
  status: "verified" | "stale" | "out_of_range" | "missing";
  total_lines: number;
  file_sha256?: string;
  line_excerpt?: string;
  baseline_sha256?: string | null;
};

export type CitationReport = {
  total: number;
  verified: number;
  stale?: number;
  broken: number;
  freshness_pct?: number;
  gate_status?: string;
  ok: boolean;
  citations: CitationCheck[];
};

export type PackPreflightReport = {
  ok: boolean;
  ready_count: number;
  empty_count: number;
  missing_count: number;
  total_bytes: number;
  citations_checked?: number;
  citations_verified?: number;
  citations_stale?: number;
  citations_broken?: number;
  citations_freshness_pct?: number;
  citation_gate?: string;
  citations?: CitationCheck[];
  files: PackPreflightFile[];
};

/** Verifies `[file:L##]` citations against live workspace files, SHA-256 digests, and line counts. */
export async function verifyCitations(
  text: string,
  opts?: { receipt_id?: string; expected_shas?: Record<string, string> }
): Promise<CitationReport> {
  const r = await fetch("/api/citations/verify", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      text,
      receipt_id: opts?.receipt_id,
      expected_shas: opts?.expected_shas,
    }),
    cache: "no-store",
  });
  if (r.ok) {
    return (await r.json()) as CitationReport;
  }
  throw new Error(`HTTP ${r.status}`);
}

/** Verifies non-empty deliverables, row/line counts, and live citation freshness before export. */
export async function preflightClientPack(
  files: string[],
  receiptId?: string
): Promise<PackPreflightReport> {
  const params = new URLSearchParams();
  if (files.length > 0) params.set("files", files.slice(0, 25).join(","));
  if (receiptId) params.set("receipt_id", receiptId);
  params.set("preflight", "1");
  const r = await fetch(`/api/files/pack?${params.toString()}`, { cache: "no-store" });
  if (r.ok) {
    return (await r.json()) as PackPreflightReport;
  }
  throw new Error(`HTTP ${r.status}`);
}

export type ReceiptCompareResult = {
  receipt_a: { id: string; task: string; summary: string; files?: string[]; sha256?: string };
  receipt_b: { id: string; task: string; summary: string; files?: string[]; sha256?: string };
  added: number;
  removed: number;
  unchanged: number;
  similarity_pct: number;
  line_delta: number;
  file_delta: number;
  files_added: string[];
  files_removed: string[];
  files_shared: string[];
  unified: string;
  delta_markdown: string;
};

/** Compares two receipts/runs side-by-side and returns line/file deltas + verified Markdown delta. */
export async function compareReceipts(a: string, b: string): Promise<ReceiptCompareResult> {
  const params = new URLSearchParams({ a, b });
  const r = await fetch(`/api/receipts/compare?${params.toString()}`, { cache: "no-store" });
  if (r.ok) {
    return (await r.json()) as ReceiptCompareResult;
  }
  throw new Error(`HTTP ${r.status}`);
}

export type PackDeltaOptions = {
  delta_a?: string;
  delta_b?: string;
  delta_file?: string;
  rev_a?: string;
  rev_b?: string;
  receipt_id?: string;
  summary?: boolean;
};

async function fetchClientPack(
  files: string[],
  title: string,
  format: "md" | "html",
  delta?: PackDeltaOptions
): Promise<{
  filename: string;
  sha256: string;
  preflight?: string;
  citationGate?: string;
  summaryCitations?: string;
  summaryStale?: number;
}> {
  const ext = format === "html" ? "html" : "md";
  const filename = format === "html" ? `ergon-${safeSlug(title)}-handoff.html` : `ergon-${safeSlug(title)}.md`;
  const mime = format === "html" ? "text/html; charset=utf-8" : "text/markdown; charset=utf-8";
  const params = new URLSearchParams();
  if (files.length > 0) params.set("files", files.slice(0, 25).join(","));
  if (title.trim()) params.set("title", title.trim());
  if (delta?.receipt_id) params.set("receipt_id", delta.receipt_id);
  if (delta?.summary) params.set("summary", "1");
  if (delta?.delta_a && delta?.delta_b) {
    params.set("delta_a", delta.delta_a);
    params.set("delta_b", delta.delta_b);
  } else if (delta?.delta_file && delta?.rev_a) {
    params.set("delta_file", delta.delta_file);
    params.set("rev_a", delta.rev_a);
    params.set("rev_b", delta.rev_b || "current");
  }
  if (format === "html") params.set("format", "html");
  params.set("download", "1");

  const r = await fetch(`/api/files/pack?${params.toString()}`, { cache: "no-store" });
  if (r.ok) {
    const text = await r.text();
    const headerHash = r.headers.get("X-Ergon-Pack-SHA256") || "";
    const preflight = r.headers.get("X-Ergon-Pack-Preflight") || undefined;
    const citationGate = r.headers.get("X-Ergon-Pack-Citation-Gate") || undefined;
    const summaryCitations = r.headers.get("X-Ergon-Pack-Summary-Citations") || undefined;
    const rawStale = r.headers.get("X-Ergon-Pack-Summary-Stale");
    const summaryStale = rawStale !== null ? Number(rawStale) : undefined;
    const digest = headerHash || (await sha256Hex(text));
    triggerDownload(filename, text, mime);
    return { filename, sha256: digest, preflight, citationGate, summaryCitations, summaryStale };
  }
  throw new Error(`HTTP ${r.status} (${ext})`);
}

/** Runs batch citation freshness pre-flight across recent finished summaries + deliverables. */
export async function fetchSummaryPreflight(
  files: string[] = []
): Promise<(PackPreflightReport & { summary_runs_count?: number }) | null> {
  const params = new URLSearchParams({ preflight: "1", summary: "1" });
  if (files.length > 0) params.set("files", files.slice(0, 25).join(","));
  try {
    const r = await fetch(`/api/files/pack?${params.toString()}`, { cache: "no-store" });
    if (!r.ok) return null;
    return (await r.json()) as PackPreflightReport & { summary_runs_count?: number };
  } catch {
    return null;
  }
}

/** Downloads a single verified Client Deliverable Pack (.md) combining selected/recent files + optional verified delta. */
export async function exportClientPackDownload(
  files: string[],
  title = "Client Deliverable Pack",
  delta?: PackDeltaOptions
): Promise<{
  filename: string;
  sha256: string;
  preflight?: string;
  citationGate?: string;
  summaryCitations?: string;
  summaryStale?: number;
}> {
  return fetchClientPack(files, title, "md", delta);
}

/** Downloads a standalone printable Verified Client Handoff (.html) with SHA-256 manifest and live citation gate. */
export async function exportClientHandoffHtml(
  files: string[],
  title = "Client Deliverable Pack",
  delta?: PackDeltaOptions
): Promise<{
  filename: string;
  sha256: string;
  preflight?: string;
  citationGate?: string;
  summaryCitations?: string;
  summaryStale?: number;
}> {
  return fetchClientPack(files, title, "html", delta);
}

/** Downloads a 1-click multi-run Retainer / Client Summary Pack (.md or .html) combining recent finished work, verified citations, and SHA-256 deliverables. */
export async function exportClientSummaryPack(
  files: string[],
  format: "md" | "html",
  title = "Client Summary Pack"
): Promise<{
  filename: string;
  sha256: string;
  preflight?: string;
  citationGate?: string;
  summaryCitations?: string;
  summaryStale?: number;
}> {
  return fetchClientPack(files, title, format, { summary: true });
}

export const PLAYBOOK_PACK_FORMAT = "ergon-playbook-v1";
export const MAX_PLAYBOOK_PACK_BYTES = 256 * 1024;

export type PlaybookPackFields = {
  id?: string | null;
  name: string;
  prompt: string;
  schedule: string;
  interval_secs?: number | null;
  skill_id?: string | null;
  steps: string[];
  expected_inputs: string[];
  pack_deliverables: boolean;
};

export type PlaybookPackDocument = {
  format: typeof PLAYBOOK_PACK_FORMAT;
  version: string;
  exported_ms: number;
  playbook: PlaybookPackFields;
  sha256: string;
};

function cleanPlaybookFieldList(raw: unknown, maxChars: number): string[] {
  if (!Array.isArray(raw)) return [];
  return raw
    .filter((v): v is string => typeof v === "string")
    .map((s) => s.trim().slice(0, maxChars))
    .filter(Boolean)
    .slice(0, 8);
}

export async function computePlaybookSha256(pb: PlaybookPackFields): Promise<string> {
  const canonical = JSON.stringify([
    PLAYBOOK_PACK_FORMAT,
    pb.name,
    pb.prompt,
    pb.schedule,
    pb.interval_secs ?? null,
    pb.skill_id ?? null,
    pb.steps,
    pb.expected_inputs,
    Boolean(pb.pack_deliverables),
  ]);
  return sha256Hex(canonical);
}

export async function buildSignedPlaybookPack(
  routine: {
    id?: string;
    name: string;
    prompt: string;
    schedule: string;
    interval_secs?: number | null;
    skill_id?: string | null;
    steps?: string[];
    expected_inputs?: string[];
    pack_deliverables?: boolean;
  },
  version = "0.20.0"
): Promise<{ doc: PlaybookPackDocument; json: string; filename: string; sha256: string }> {
  const name = String(routine.name || "").replace(/[\r\n]/g, " ").trim().slice(0, 120);
  const prompt = String(routine.prompt || "").trim().slice(0, 4000);
  const schedule = String(routine.schedule || "manual").replace(/[\r\n]/g, " ").trim().slice(0, 64) || "manual";
  const interval_secs = typeof routine.interval_secs === "number" && Number.isInteger(routine.interval_secs) && routine.interval_secs >= 0
    ? routine.interval_secs
    : null;
  const skill_id = typeof routine.skill_id === "string" && routine.skill_id.trim()
    ? routine.skill_id.trim().slice(0, 64)
    : null;
  const steps = cleanPlaybookFieldList(routine.steps, 2000);
  const expected_inputs = cleanPlaybookFieldList(routine.expected_inputs, 64);
  const pack_deliverables = Boolean(routine.pack_deliverables);
  const fields: PlaybookPackFields = {
    id: routine.id ?? null,
    name,
    prompt,
    schedule,
    interval_secs,
    skill_id,
    steps,
    expected_inputs,
    pack_deliverables,
  };
  const sha256 = await computePlaybookSha256(fields);
  const doc: PlaybookPackDocument = {
    format: PLAYBOOK_PACK_FORMAT,
    version,
    exported_ms: Date.now(),
    playbook: fields,
    sha256,
  };
  const slug = safeSlug(name) || "playbook";
  return {
    doc,
    json: JSON.stringify(doc, null, 2),
    filename: `${slug}.ergon-playbook.json`,
    sha256,
  };
}

export async function verifyAndParsePlaybookPack(rawBytesOrText: string): Promise<{
  ok: true;
  playbook: PlaybookPackFields;
  sha256: string;
} | {
  ok: false;
  status: 400 | 413;
  error: string;
}> {
  const byteLen = new TextEncoder().encode(rawBytesOrText).length;
  if (byteLen > MAX_PLAYBOOK_PACK_BYTES) {
    return {
      ok: false,
      status: 413,
      error: `playbook pack exceeds maximum size of ${MAX_PLAYBOOK_PACK_BYTES} bytes`,
    };
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(rawBytesOrText);
  } catch (e) {
    return { ok: false, status: 400, error: `invalid playbook JSON: ${String(e)}` };
  }
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    return { ok: false, status: 400, error: "playbook pack must be a JSON object" };
  }
  const top = parsed as Record<string, unknown>;
  const allowedTop = new Set(["format", "version", "exported_ms", "playbook", "sha256"]);
  for (const key of Object.keys(top)) {
    if (!allowedTop.has(key)) return { ok: false, status: 400, error: `unexpected top-level field '${key}'` };
  }
  if (top.format !== PLAYBOOK_PACK_FORMAT) {
    return { ok: false, status: 400, error: `playbook.format must be '${PLAYBOOK_PACK_FORMAT}'` };
  }
  if (top.version !== undefined && top.version !== null) {
    if (typeof top.version !== "string" || top.version.length > 32) {
      return { ok: false, status: 400, error: "version must be a string <= 32 chars" };
    }
  }
  if (top.exported_ms !== undefined && top.exported_ms !== null) {
    if (typeof top.exported_ms !== "number" || !Number.isInteger(top.exported_ms) || top.exported_ms < 0) {
      return { ok: false, status: 400, error: "exported_ms must be an unsigned integer" };
    }
  }
  const claimedSha = typeof top.sha256 === "string" ? top.sha256.trim() : "";
  if (!/^[a-fA-F0-9]{64}$/.test(claimedSha)) {
    return { ok: false, status: 400, error: "sha256 checksum must be a 64-character hex digest" };
  }
  const pbRaw = top.playbook;
  if (!pbRaw || typeof pbRaw !== "object" || Array.isArray(pbRaw)) {
    return { ok: false, status: 400, error: "playbook object is required" };
  }
  const pb = pbRaw as Record<string, unknown>;
  const allowedPb = new Set(["id", "name", "prompt", "schedule", "interval_secs", "skill_id", "steps", "expected_inputs", "pack_deliverables"]);
  for (const key of Object.keys(pb)) {
    if (!allowedPb.has(key)) return { ok: false, status: 400, error: `unexpected playbook field '${key}'` };
  }
  if (pb.id !== undefined && pb.id !== null && (typeof pb.id !== "string" || pb.id.length > 128)) {
    return { ok: false, status: 400, error: "playbook.id must be a string <= 128 chars" };
  }
  if (typeof pb.name !== "string" || !pb.name.trim() || /[\r\n]/.test(pb.name) || pb.name.length > 120) {
    return { ok: false, status: 400, error: "playbook.name must be 1..=120 single-line characters" };
  }
  if (typeof pb.prompt !== "string" || !pb.prompt.trim() || pb.prompt.length > 4000) {
    return { ok: false, status: 400, error: "playbook.prompt must be 1..=4000 characters" };
  }
  if (typeof pb.schedule !== "string" || !pb.schedule.trim() || /[\r\n]/.test(pb.schedule) || pb.schedule.length > 64) {
    return { ok: false, status: 400, error: "playbook.schedule must be 1..=64 single-line characters" };
  }
  let intervalSecs: number | null = null;
  if (pb.interval_secs !== undefined && pb.interval_secs !== null) {
    if (typeof pb.interval_secs !== "number" || !Number.isInteger(pb.interval_secs) || pb.interval_secs < 10 || pb.interval_secs > 2_592_000) {
      return { ok: false, status: 400, error: "playbook.interval_secs must be an integer between 10 and 2592000" };
    }
    intervalSecs = pb.interval_secs;
  }
  let skillId: string | null = null;
  if (pb.skill_id !== undefined && pb.skill_id !== null) {
    if (typeof pb.skill_id !== "string" || !pb.skill_id.trim() || pb.skill_id.length > 64) {
      return { ok: false, status: 400, error: "playbook.skill_id must be 1..=64 characters" };
    }
    skillId = pb.skill_id;
  }
  const checkArr = (val: unknown, key: string, maxChars: number): string[] | string => {
    if (val === undefined || val === null) return [];
    if (!Array.isArray(val) || val.length > 8) return `playbook.${key} must be an array of at most 8 strings`;
    for (let i = 0; i < val.length; i += 1) {
      const item = val[i];
      if (typeof item !== "string" || !item.trim() || item.length > maxChars) {
        return `playbook.${key}[${i}] must be 1..=${maxChars} characters`;
      }
    }
    return val as string[];
  };
  const steps = checkArr(pb.steps, "steps", 2000);
  if (typeof steps === "string") return { ok: false, status: 400, error: steps };
  const expectedInputs = checkArr(pb.expected_inputs, "expected_inputs", 64);
  if (typeof expectedInputs === "string") return { ok: false, status: 400, error: expectedInputs };
  let packDeliverables = false;
  if (pb.pack_deliverables !== undefined && pb.pack_deliverables !== null) {
    if (typeof pb.pack_deliverables !== "boolean") {
      return { ok: false, status: 400, error: "playbook.pack_deliverables must be a boolean" };
    }
    packDeliverables = pb.pack_deliverables;
  }
  const fields: PlaybookPackFields = {
    id: (pb.id as string | null | undefined) ?? null,
    name: pb.name,
    prompt: pb.prompt,
    schedule: pb.schedule,
    interval_secs: intervalSecs,
    skill_id: skillId,
    steps,
    expected_inputs: expectedInputs,
    pack_deliverables: packDeliverables,
  };
  const expectedSha = await computePlaybookSha256(fields);
  if (expectedSha.toLowerCase() !== claimedSha.toLowerCase()) {
    return {
      ok: false,
      status: 400,
      error: "sha256 checksum mismatch: playbook pack failed integrity verification",
    };
  }
  return { ok: true, playbook: fields, sha256: expectedSha };
}

/** Downloads a signed .ergon-playbook.json pack with SHA-256 verification metadata. */
export async function exportPlaybookDownload(
  routine: {
    id: string;
    name: string;
    prompt: string;
    schedule: string;
    interval_secs?: number | null;
    skill_id?: string | null;
    steps?: string[];
    expected_inputs?: string[];
    pack_deliverables?: boolean;
  },
  useServer: boolean
): Promise<{ filename: string; sha256: string }> {
  const fallbackSlug = safeSlug(routine.name) || "playbook";
  const fallbackName = `${fallbackSlug}.ergon-playbook.json`;
  if (useServer && routine.id) {
    try {
      const r = await fetch(`/api/playbooks/${encodeURIComponent(routine.id)}/export`, { cache: "no-store" });
      if (r.ok) {
        const text = await r.text();
        const headerHash = r.headers.get("X-Ergon-Playbook-SHA256") || "";
        let parsedSha = "";
        try {
          const parsed = JSON.parse(text) as { sha256?: string };
          parsedSha = typeof parsed.sha256 === "string" ? parsed.sha256 : "";
        } catch { /* ignore */ }
        const disp = r.headers.get("Content-Disposition") || "";
        const m = disp.match(/filename="([^"]+)"/);
        const filename = m?.[1] || fallbackName;
        const sha256 = headerHash || parsedSha || (await sha256Hex(text));
        triggerDownload(filename, text, "application/json; charset=utf-8");
        return { filename, sha256 };
      }
    } catch {
      // Fall back to local deterministic signing.
    }
  }
  const built = await buildSignedPlaybookPack(routine);
  triggerDownload(built.filename, built.json, "application/json; charset=utf-8");
  return { filename: built.filename, sha256: built.sha256 };
}

function inferPreviewStep(
  stepText: string,
  fallbackSkill: string | undefined,
  isFirst: boolean
): { skillId: string | null; mode: "read_only" | "read_and_write"; outputHint: string | null } {
  const lower = stepText.toLowerCase();
  const explicitSkills = [
    "reconcile_ledger", "clean_dataset", "audit_contract", "summarize_meeting",
    "triage_inbox", "compare_documents", "draft_brief", "extract_tables",
    "make_spreadsheet", "write_document", "update_memory", "research",
  ];
  let matched: string | null = explicitSkills.find((id) => lower.includes(id)) ?? null;
  if (!matched) {
    if (/ledger|reconcil|expense/.test(lower)) matched = "reconcile_ledger";
    else if (/clean|dataset|normaliz/.test(lower)) matched = "clean_dataset";
    else if (/contract|clause/.test(lower)) matched = "audit_contract";
    else if (/meeting|action item/.test(lower)) matched = "summarize_meeting";
    else if (/inbox|triage|email/.test(lower)) matched = "triage_inbox";
    else if (/compare|diff/.test(lower)) matched = "compare_documents";
    else if (/spreadsheet|\.xlsx/.test(lower)) matched = "make_spreadsheet";
    else if (/docx|word|memo|write/.test(lower)) matched = "write_document";
    else if (/table|extract/.test(lower)) matched = "extract_tables";
    else if (/brief|summary/.test(lower)) matched = "draft_brief";
    else if (/memory/.test(lower)) matched = "update_memory";
    else if (/research/.test(lower)) matched = "research";
    else if (isFirst && fallbackSkill) matched = fallbackSkill;
  }
  const writes = /export|write|save|create|clean|\.docx|\.xlsx/.test(lower)
    || ["clean_dataset", "reconcile_ledger", "make_spreadsheet", "write_document", "draft_brief", "update_memory"].includes(matched ?? "");
  const outputMap: Record<string, string> = {
    clean_dataset: "Cleaned dataset (.csv)",
    reconcile_ledger: "Expense reconciliation report (.md)",
    make_spreadsheet: "Typed workbook (.xlsx)",
    write_document: "Formatted deliverable (.docx / .md)",
    draft_brief: "Executive brief (.md)",
    summarize_meeting: "Meeting digest & follow-up (.md)",
    update_memory: "Standing memory rules (memory.md)",
  };
  let outputHint: string | null = matched ? (outputMap[matched] ?? null) : null;
  if (matched === "audit_contract" && writes) outputHint = "Contract audit findings (.md)";
  if (!outputHint && writes) outputHint = "Workspace deliverable artifact";
  return { skillId: matched, mode: writes ? "read_and_write" : "read_only", outputHint };
}

/** Builds a deterministic zero-side-effect Dry-Run Preview matching POST /api/playbooks/:id/preview. */
export function buildDeterministicPlaybookPreview(
  routine: {
    id: string;
    name: string;
    prompt: string;
    schedule: string;
    skill_id?: string;
    steps?: string[];
    expected_inputs?: string[];
    pack_deliverables?: boolean;
  },
  workspacePaths: string[],
  revisionCountFor: (path: string) => number = () => 0,
  totalTrackedFiles = 0
): import("@/lib/api").PlaybookPreviewResult {
  const exts = [".csv", ".tsv", ".md", ".markdown", ".txt", ".json", ".xlsx", ".docx", ".pdf", ".pptx", ".yaml", ".log"];
  const rawSteps = routine.steps && routine.steps.length > 0
    ? routine.steps.map((s) => s.trim()).filter(Boolean)
    : [routine.prompt.trim()].filter(Boolean);
  const expectedInputs = (routine.expected_inputs ?? []).map((s) => s.trim()).filter(Boolean);
  const packDeliverables = Boolean(routine.pack_deliverables);
  const impactMap = new Map<string, { role: string; step: string }>();
  const pushImpact = (path: string, role: string, step: string) => {
    const cur = impactMap.get(path);
    if (!cur) impactMap.set(path, { role, step });
    else if (role === "read_and_update" || role === "deliverable_output") cur.role = role;
  };

  const stepsOut = rawSteps.map((stepText, idx) => {
    const { skillId, mode, outputHint } = inferPreviewStep(stepText, routine.skill_id, idx === 0);
    const refs: string[] = [];
    const addRef = (cand: string) => {
      const clean = cand
        .replace(/^["'`(\[{]+|["'`)\]},;.]+$/g, "")
        .replace(/:L\d+(?:-\d+)?$/i, "");
      if (!clean || clean.startsWith("/") || clean.includes("..")) return;
      if (!refs.includes(clean)) refs.push(clean);
    };
    if (idx === 0) expectedInputs.forEach(addRef);
    for (const tok of stepText.split(/\s+/)) {
      const low = tok.toLowerCase();
      if (exts.some((ext) => low.endsWith(ext) || low.includes(`${ext}:l`) || low.includes(`${ext},`))) {
        addRef(tok);
      }
    }
    const lowStep = stepText.toLowerCase();
    for (const wsPath of workspacePaths) {
      const base = wsPath.split("/").pop() || wsPath;
      if ((lowStep.includes(wsPath.toLowerCase()) || lowStep.includes(base.toLowerCase())) && !refs.includes(wsPath)) {
        refs.push(wsPath);
      }
    }
    const predictedOutputs: string[] = [];
    if (outputHint) predictedOutputs.push(outputHint);
    if (packDeliverables && idx + 1 === rawSteps.length) predictedOutputs.push("Playbook Deliverable Pack.md");
    for (const r of refs) {
      const role = mode === "read_and_write" && r.endsWith(".csv") && skillId === "clean_dataset" ? "read_and_update" : "input";
      pushImpact(r, role, `Step ${idx + 1}`);
    }
    return {
      step: idx + 1,
      prompt: stepText,
      skill_id: skillId,
      mode,
      referenced_paths: refs,
      predicted_outputs: predictedOutputs,
    };
  });

  if (packDeliverables) {
    pushImpact("Playbook Deliverable Pack.md", "deliverable_pack", "Final step");
  }
  if (impactMap.size === 0) {
    workspacePaths.slice(0, 3).forEach((ws) => pushImpact(ws, "workspace_context", "Context"));
  }

  let existingSnapshots = totalTrackedFiles;
  const fileImpacts = Array.from(impactMap.entries()).map(([path, info]) => {
    const exists = workspacePaths.includes(path);
    const revs = revisionCountFor(path);
    if (revs > existingSnapshots) existingSnapshots = revs;
    return {
      path,
      role: info.role,
      step: info.step,
      exists_in_workspace: exists,
      history_revisions: revs,
      reversible: true,
    };
  });

  return {
    routine_id: routine.id,
    name: routine.name,
    schedule: routine.schedule,
    dry_run: true,
    zero_side_effects: true,
    step_count: stepsOut.length,
    steps: stepsOut,
    expected_inputs: expectedInputs,
    matched_workspace_files: workspacePaths.filter((p) => impactMap.has(p)),
    file_impacts: fileImpacts,
    pack_deliverables: packDeliverables,
    rollback: {
      safe: true,
      badge: "Reversible via .history/",
      history_snapshots_enabled: true,
      existing_snapshots: existingSnapshots,
      reversible_paths: fileImpacts.length,
      note: "Dry-run preview inspects your workspace read-only. Every existing file modified during execution is snapshotted into .history/ first for 1-click restore.",
    },
    generated_ms: Date.now(),
  };
}
