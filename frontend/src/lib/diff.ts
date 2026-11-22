// Line-by-line unified diff engine + defensive normalizer for server /api/files/history diff payloads.
import type { FileDiffHunk, FileDiffLine, FileDiffPayload } from "@/lib/api";

const MAX_LCS_CELLS = 400_000;

function splitLines(text: string): string[] {
  if (!text) return [];
  return text.replace(/\r\n/g, "\n").split("\n");
}

function buildHunks(ops: FileDiffLine[], context = 3): FileDiffHunk[] {
  const changeIdxs: number[] = [];
  ops.forEach((op, i) => {
    if (op.kind === "add" || op.kind === "del") changeIdxs.push(i);
  });
  if (!changeIdxs.length) return [];

  const ranges: [number, number][] = [];
  for (const idx of changeIdxs) {
    const start = Math.max(0, idx - context);
    const end = Math.min(ops.length - 1, idx + context);
    if (ranges.length && start <= ranges[ranges.length - 1][1] + 1) {
      ranges[ranges.length - 1][1] = Math.max(ranges[ranges.length - 1][1], end);
    } else {
      ranges.push([start, end]);
    }
  }

  return ranges.map(([start, end]) => {
    const slice = ops.slice(start, end + 1);
    const firstOld = slice.find((l) => typeof l.old_line === "number")?.old_line ?? 1;
    const firstNew = slice.find((l) => typeof l.new_line === "number")?.new_line ?? 1;
    const oldCount = slice.filter((l) => l.kind !== "add").length;
    const newCount = slice.filter((l) => l.kind !== "del").length;
    return {
      header: `@@ -${firstOld},${oldCount} +${firstNew},${newCount} @@`,
      old_start: firstOld,
      old_count: oldCount,
      new_start: firstNew,
      new_count: newCount,
      lines: slice,
    };
  });
}

export function computeLineDiff(
  beforeText: string,
  afterText: string,
  meta?: { path?: string; rev_a?: string; rev_b?: string }
): FileDiffPayload {
  const a = splitLines(beforeText);
  const b = splitLines(afterText);

  // Trim identical prefix & suffix before DP to keep large files fast.
  let prefix = 0;
  const minLen = Math.min(a.length, b.length);
  while (prefix < minLen && a[prefix] === b[prefix]) prefix++;

  let suffix = 0;
  while (
    suffix < minLen - prefix &&
    a[a.length - 1 - suffix] === b[b.length - 1 - suffix]
  ) {
    suffix++;
  }

  const midA = a.slice(prefix, a.length - suffix);
  const midB = b.slice(prefix, b.length - suffix);

  const ops: FileDiffLine[] = [];
  let oldLn = 1;
  let newLn = 1;

  for (let i = 0; i < prefix; i++) {
    ops.push({ kind: "context", text: a[i], old_line: oldLn++, new_line: newLn++ });
  }

  if (midA.length * midB.length <= MAX_LCS_CELLS) {
    const m = midA.length;
    const n = midB.length;
    const stride = n + 1;
    const dp = new Uint32Array((m + 1) * stride);
    for (let i = m - 1; i >= 0; i--) {
      for (let j = n - 1; j >= 0; j--) {
        dp[i * stride + j] =
          midA[i] === midB[j]
            ? dp[(i + 1) * stride + (j + 1)] + 1
            : Math.max(dp[(i + 1) * stride + j], dp[i * stride + (j + 1)]);
      }
    }
    let i = 0;
    let j = 0;
    while (i < m && j < n) {
      if (midA[i] === midB[j]) {
        ops.push({ kind: "context", text: midA[i], old_line: oldLn++, new_line: newLn++ });
        i++;
        j++;
      } else if (dp[(i + 1) * stride + j] >= dp[i * stride + (j + 1)]) {
        ops.push({ kind: "del", text: midA[i], old_line: oldLn++, new_line: null });
        i++;
      } else {
        ops.push({ kind: "add", text: midB[j], old_line: null, new_line: newLn++ });
        j++;
      }
    }
    while (i < m) ops.push({ kind: "del", text: midA[i++], old_line: oldLn++, new_line: null });
    while (j < n) ops.push({ kind: "add", text: midB[j++], old_line: null, new_line: newLn++ });
  } else {
    // Safe O(N) fallback for unusually large diffs.
    for (const line of midA) ops.push({ kind: "del", text: line, old_line: oldLn++, new_line: null });
    for (const line of midB) ops.push({ kind: "add", text: line, old_line: null, new_line: newLn++ });
  }

  for (let i = a.length - suffix; i < a.length; i++) {
    ops.push({ kind: "context", text: a[i], old_line: oldLn++, new_line: newLn++ });
  }

  let added = 0;
  let removed = 0;
  let unchanged = 0;
  for (const op of ops) {
    if (op.kind === "add") added++;
    else if (op.kind === "del") removed++;
    else unchanged++;
  }

  const hunks = buildHunks(ops, 3);
  const unifiedLines: string[] = [];
  if (meta?.path) {
    unifiedLines.push(`--- ${meta.path}@${meta.rev_a || "previous"}`);
    unifiedLines.push(`+++ ${meta.path}@${meta.rev_b || "current"}`);
  }
  for (const h of hunks) {
    if (h.header) unifiedLines.push(h.header);
    for (const l of h.lines) {
      const prefixCh = l.kind === "add" ? "+" : l.kind === "del" ? "-" : " ";
      unifiedLines.push(prefixCh + l.text);
    }
  }

  return {
    path: meta?.path,
    rev_a: meta?.rev_a,
    rev_b: meta?.rev_b,
    added,
    removed,
    unchanged,
    hunks,
    unified: unifiedLines.join("\n"),
  };
}

function normalizeDiffLine(raw: unknown): FileDiffLine | null {
  if (typeof raw === "string") {
    if (raw.startsWith("@@")) return { kind: "hunk", text: raw };
    if (raw.startsWith("+")) return { kind: "add", text: raw.slice(1) };
    if (raw.startsWith("-")) return { kind: "del", text: raw.slice(1) };
    if (raw.startsWith(" ")) return { kind: "context", text: raw.slice(1) };
    return { kind: "context", text: raw };
  }
  if (!raw || typeof raw !== "object") return null;
  const o = raw as Record<string, unknown>;
  const tag = String(o.kind ?? o.tag ?? o.op ?? o.type ?? "context").toLowerCase();
  const kind: FileDiffLine["kind"] =
    tag === "add" || tag === "added" || tag === "insert" || tag === "+"
      ? "add"
      : tag === "del" || tag === "delete" || tag === "removed" || tag === "remove" || tag === "-"
        ? "del"
        : tag === "hunk"
          ? "hunk"
          : "context";
  const text = String(o.text ?? o.line ?? o.content ?? "");
  return {
    kind,
    text,
    old_line:
      typeof o.left_line === "number"
        ? o.left_line
        : typeof o.old_line === "number"
          ? o.old_line
          : typeof o.old_lineno === "number"
            ? o.old_lineno
            : null,
    new_line:
      typeof o.right_line === "number"
        ? o.right_line
        : typeof o.new_line === "number"
          ? o.new_line
          : typeof o.new_lineno === "number"
            ? o.new_lineno
            : null,
  };
}

export function parseUnifiedString(unified: string): FileDiffPayload {
  const lines = unified.replace(/\r\n/g, "\n").split("\n");
  const hunks: FileDiffHunk[] = [];
  let cur: FileDiffHunk | null = null;
  let oldLn = 1;
  let newLn = 1;
  let added = 0;
  let removed = 0;
  let unchanged = 0;

  for (const raw of lines) {
    if (raw.startsWith("--- ") || raw.startsWith("+++ ")) continue;
    const hm = /^@@\s+-(\d+)(?:,(\d+))?\s+\+(\d+)(?:,(\d+))?\s+@@/.exec(raw);
    if (hm) {
      oldLn = Number(hm[1]);
      newLn = Number(hm[3]);
      cur = {
        header: raw,
        old_start: oldLn,
        old_count: hm[2] ? Number(hm[2]) : 1,
        new_start: newLn,
        new_count: hm[4] ? Number(hm[4]) : 1,
        lines: [],
      };
      hunks.push(cur);
      continue;
    }
    if (!cur) {
      if (!raw) continue;
      cur = { header: "@@ -1 +1 @@", old_start: 1, new_start: 1, lines: [] };
      hunks.push(cur);
    }
    if (raw.startsWith("+")) {
      added++;
      cur.lines.push({ kind: "add", text: raw.slice(1), old_line: null, new_line: newLn++ });
    } else if (raw.startsWith("-")) {
      removed++;
      cur.lines.push({ kind: "del", text: raw.slice(1), old_line: oldLn++, new_line: null });
    } else {
      unchanged++;
      cur.lines.push({
        kind: "context",
        text: raw.startsWith(" ") ? raw.slice(1) : raw,
        old_line: oldLn++,
        new_line: newLn++,
      });
    }
  }

  return { added, removed, unchanged, hunks, unified };
}

/** Normalizes any server diff payload shape (structured hunks, flat lines, or unified diff text). */
export function normalizeServerDiff(raw: unknown): FileDiffPayload | null {
  if (!raw) return null;
  if (typeof raw === "string") return parseUnifiedString(raw);
  if (typeof raw !== "object") return null;
  const o = raw as Record<string, unknown>;
  const target = (o.diff && typeof o.diff === "object" ? o.diff : o) as Record<string, unknown>;

  if (typeof target.unified === "string" && !Array.isArray(target.hunks) && !Array.isArray(target.lines)) {
    const parsed = parseUnifiedString(target.unified);
    return {
      ...parsed,
      added: typeof target.added === "number" ? target.added : typeof target.additions === "number" ? target.additions : parsed.added,
      removed: typeof target.removed === "number" ? target.removed : typeof target.deletions === "number" ? target.deletions : parsed.removed,
    };
  }

  const hunks: FileDiffHunk[] = [];
  if (Array.isArray(target.hunks)) {
    for (const h of target.hunks) {
      if (!h || typeof h !== "object") continue;
      const ho = h as Record<string, unknown>;
      const rawLines = Array.isArray(ho.lines) ? ho.lines : [];
      const normLines = rawLines.map(normalizeDiffLine).filter((x): x is FileDiffLine => x !== null);
      const oldStart = typeof ho.left_start === "number" ? ho.left_start : typeof ho.old_start === "number" ? ho.old_start : undefined;
      const newStart = typeof ho.right_start === "number" ? ho.right_start : typeof ho.new_start === "number" ? ho.new_start : undefined;
      const oldCount = typeof ho.left_count === "number" ? ho.left_count : typeof ho.old_count === "number" ? ho.old_count : undefined;
      const newCount = typeof ho.right_count === "number" ? ho.right_count : typeof ho.new_count === "number" ? ho.new_count : undefined;
      let oLn = oldStart ?? 1;
      let nLn = newStart ?? 1;
      const numbered = normLines.map((l) => {
        if (l.kind === "add") {
          const ln = l.new_line ?? nLn++;
          return { ...l, old_line: null, new_line: ln };
        }
        if (l.kind === "del") {
          const ln = l.old_line ?? oLn++;
          return { ...l, old_line: ln, new_line: null };
        }
        const ol = l.old_line ?? oLn++;
        const nl = l.new_line ?? nLn++;
        return { ...l, old_line: ol, new_line: nl };
      });
      hunks.push({
        header:
          typeof ho.header === "string"
            ? ho.header
            : oldStart !== undefined && newStart !== undefined
              ? `@@ -${oldStart},${oldCount ?? numbered.filter((x) => x.kind !== "add").length} +${newStart},${newCount ?? numbered.filter((x) => x.kind !== "del").length} @@`
              : undefined,
        old_start: oldStart,
        old_count: oldCount,
        new_start: newStart,
        new_count: newCount,
        lines: numbered,
      });
    }
  } else if (Array.isArray(target.lines)) {
    const normLines = target.lines.map(normalizeDiffLine).filter((x): x is FileDiffLine => x !== null);
    if (normLines.length) hunks.push({ header: "@@ -1 +1 @@", lines: normLines });
  } else {
    return null;
  }

  let added = 0;
  let removed = 0;
  let unchanged = 0;
  for (const h of hunks) {
    for (const l of h.lines) {
      if (l.kind === "add") added++;
      else if (l.kind === "del") removed++;
      else if (l.kind === "context") unchanged++;
    }
  }

  return {
    path: typeof target.path === "string" ? target.path : undefined,
    rev_a: typeof target.rev_a === "string" ? target.rev_a : undefined,
    rev_b: typeof target.rev_b === "string" ? target.rev_b : undefined,
    added: typeof target.added === "number" ? target.added : typeof target.additions === "number" ? target.additions : added,
    removed: typeof target.removed === "number" ? target.removed : typeof target.deletions === "number" ? target.deletions : removed,
    unchanged: typeof target.unchanged === "number" ? target.unchanged : unchanged,
    hunks,
    unified: typeof target.unified === "string" ? target.unified : undefined,
  };
}
