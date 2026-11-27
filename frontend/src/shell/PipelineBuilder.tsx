import { useEffect, useMemo, useRef, useState } from "react";
import {
  AlertTriangle,
  ArrowDown,
  ArrowUp,
  Bot,
  CheckCircle2,
  FileDown,
  GitBranch,
  Layers,
  PauseCircle,
  Plus,
  Send,
  Trash2,
  Wrench,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  listConnectors,
  validatePipeline,
  type OutboundConnector,
  type PipelineCheckpointInputMode,
  type PipelineOnError,
  type PipelineStepKind,
  type PipelineStepSpec,
  type PipelineValidationResult as ApiPipelineValidationResult,
  type PipelineWhen,
  type Routine,
  type WorkspaceFile,
} from "@/lib/api";
import type { SkillMeta } from "@/lib/skillsCatalog";
import { cn } from "@/lib/utils";

const KIND_META: Record<
  PipelineStepKind,
  { label: string; badge: string; icon: typeof Wrench }
> = {
  skill: { label: "Skill", badge: "Deterministic", icon: Wrench },
  agent: { label: "AI Agent", badge: "Model step", icon: Bot },
  checkpoint: { label: "Human Checkpoint", badge: "Pause & resume", icon: PauseCircle },
  export: { label: "Export", badge: "Deliverable", icon: FileDown },
  deliver: { label: "Deliver", badge: "Outbound webhook", icon: Send },
  subpipeline: { label: "Subpipeline", badge: "Nested routine", icon: Layers },
};

function sanitizeStepId(raw: string, fallback: string): string {
  const cleaned = raw
    .toLowerCase()
    .replace(/[^a-z0-9_]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .slice(0, 32);
  return cleaned || fallback;
}

export type PipelineValidationResult = {
  valid: boolean;
  errorsByStep: Record<number, string[]>;
  globalErrors: string[];
  topologicalOrder: string[];
  stages: string[][];
};

function collectAncestors(stepId: string, steps: PipelineStepSpec[]): Set<string> {
  const byId = new Map<string, PipelineStepSpec>();
  for (const s of steps) {
    const id = String(s.id ?? "").trim();
    if (id) byId.set(id, s);
  }
  const visited = new Set<string>();
  const stack = [...(byId.get(stepId)?.depends_on ?? [])];
  while (stack.length > 0) {
    const cur = String(stack.pop() ?? "").trim();
    if (!cur || visited.has(cur)) continue;
    visited.add(cur);
    const parent = byId.get(cur);
    if (parent?.depends_on) {
      for (const dep of parent.depends_on) stack.push(dep);
    }
  }
  return visited;
}

function computeTopologicalStages(steps: PipelineStepSpec[]): {
  hasCycle: boolean;
  cycleIds: string[];
  topologicalOrder: string[];
  stages: string[][];
  missingRefsByStep: Record<number, string[]>;
} {
  const idToIndex = new Map<string, number>();
  steps.forEach((s, idx) => {
    const id = String(s.id ?? "").trim();
    if (id && !idToIndex.has(id)) idToIndex.set(id, idx);
  });

  const inDegree = new Map<string, number>();
  const dependents = new Map<string, string[]>();
  for (const id of idToIndex.keys()) {
    inDegree.set(id, 0);
    dependents.set(id, []);
  }

  const missingRefsByStep: Record<number, string[]> = {};
  steps.forEach((st, idx) => {
    const id = String(st.id ?? "").trim();
    const deps = Array.isArray(st.depends_on) ? st.depends_on : [];
    const seen = new Set<string>();
    const errs: string[] = [];
    for (const rawDep of deps) {
      const dep = String(rawDep ?? "").trim();
      if (!dep) continue;
      if (dep === id) {
        errs.push(`Step "${id}" cannot depend on itself.`);
        continue;
      }
      if (!idToIndex.has(dep)) {
        errs.push(`Depends on unknown step "${dep}".`);
        continue;
      }
      if (seen.has(dep)) continue;
      seen.add(dep);
      if (id) {
        dependents.get(dep)!.push(id);
        inDegree.set(id, (inDegree.get(id) ?? 0) + 1);
      }
    }
    if (errs.length > 0) missingRefsByStep[idx] = errs;
  });

  const stages: string[][] = [];
  const topologicalOrder: string[] = [];
  let frontier = steps
    .map((s) => String(s.id ?? "").trim())
    .filter((id, idx, arr) => id && arr.indexOf(id) === idx && (inDegree.get(id) ?? 0) === 0);

  while (frontier.length > 0) {
    stages.push(frontier);
    const next: string[] = [];
    for (const id of frontier) {
      topologicalOrder.push(id);
      for (const child of dependents.get(id) ?? []) {
        const deg = (inDegree.get(child) ?? 0) - 1;
        inDegree.set(child, deg);
        if (deg === 0) next.push(child);
      }
    }
    frontier = next;
  }

  const cycleIds = Array.from(idToIndex.keys()).filter((id) => !topologicalOrder.includes(id));
  return {
    hasCycle: cycleIds.length > 0,
    cycleIds,
    topologicalOrder,
    stages,
    missingRefsByStep,
  };
}

function validateTemplateText(
  raw: string,
  stepIndex: number,
  steps: PipelineStepSpec[],
  hasDagDeps: boolean
): string[] {
  const errs: string[] = [];
  const text = String(raw || "");
  if (!text.includes("{{") && !text.includes("}}")) return errs;

  const curStepId = String(steps[stepIndex]?.id ?? "").trim();
  const ancestors = hasDagDeps && curStepId ? collectAncestors(curStepId, steps) : new Set<string>();

  let cursor = 0;
  while (cursor < text.length) {
    const open = text.indexOf("{{", cursor);
    const strayClose = text.indexOf("}}", cursor);
    if (strayClose !== -1 && (open === -1 || strayClose < open)) {
      errs.push("Unmatched closing }} in template expression.");
      break;
    }
    if (open === -1) break;
    const close = text.indexOf("}}", open + 2);
    if (close === -1) {
      errs.push("Unclosed {{ template expression.");
      break;
    }
    const expr = text.slice(open + 2, close).trim();
    if (!expr) {
      errs.push("Empty {{}} template expression is not allowed.");
    } else if (expr.includes("{{")) {
      errs.push(`Nested template syntax near "{{${expr}".`);
    } else if (expr === "item") {
      // Valid inside for_each step templates
    } else if (expr.startsWith("inputs.")) {
      const key = expr.slice("inputs.".length).trim();
      if (!key || !/^[a-zA-Z0-9_.-]+$/.test(key)) {
        errs.push(`Invalid input template "{{${expr}}}".`);
      }
    } else if (expr.startsWith("steps.")) {
      const rest = expr.slice("steps.".length);
      const dot = rest.indexOf(".");
      if (dot <= 0 || dot === rest.length - 1) {
        errs.push(`Incomplete step reference "{{${expr}}}". Use {{steps.<id>.output}} or {{steps.<id>.files[0]}}.`);
      } else {
        const refId = rest.slice(0, dot);
        const field = rest.slice(dot + 1);
        const validField =
          field === "files" ||
          field === "items" ||
          /^files\[\d+\]$/.test(field) ||
          field === "output" ||
          field === "human_answer" ||
          /^metrics\.[a-zA-Z0-9_.-]+$/.test(field);
        if (!validField) {
          errs.push(`Unknown step field "${field}" in "{{${expr}}}".`);
        }
        const targetIdx = steps.findIndex((s) => s.id.trim() === refId);
        if (targetIdx < 0) {
          errs.push(`Unknown step reference "${refId}" in "{{${expr}}}".`);
        } else if (targetIdx === stepIndex) {
          errs.push(`Step "${refId}" cannot reference itself in "{{${expr}}}".`);
        } else if (!hasDagDeps && targetIdx > stepIndex) {
          errs.push(`Step cannot reference later step "${refId}" in "{{${expr}}}".`);
        } else if (hasDagDeps && ancestors.size > 0 && !ancestors.has(refId) && targetIdx > stepIndex) {
          errs.push(`Step "${refId}" must precede or be in depends_on for "{{${expr}}}".`);
        }
      }
    } else {
      errs.push(`Unsupported template "{{${expr}}}". Use {{inputs.*}} or {{steps.<prior_step>.*}}.`);
    }
    cursor = close + 2;
  }
  return errs;
}

/** Validate typed pipeline steps for empty prompts, duplicate IDs, DAG cycles, and broken templates. */
export function validatePipelineSteps(steps: PipelineStepSpec[]): PipelineValidationResult {
  const errorsByStep: Record<number, string[]> = {};
  const globalErrors: string[] = [];
  if (steps.length === 0) {
    globalErrors.push("Add at least one pipeline step.");
    return { valid: false, errorsByStep, globalErrors, topologicalOrder: [], stages: [] };
  }

  const hasDagDeps = steps.some((s) => Array.isArray(s.depends_on) && s.depends_on.length > 0);
  const topo = computeTopologicalStages(steps);
  if (topo.hasCycle) {
    globalErrors.push(`Circular dependency detected involving: ${topo.cycleIds.join(", ")}.`);
  }

  const seenIds = new Map<string, number>();
  steps.forEach((st, idx) => {
    const stepErrs: string[] = [...(topo.missingRefsByStep[idx] ?? [])];
    const id = String(st.id ?? "").trim();
    if (!id) {
      stepErrs.push("Step ID is required.");
    } else {
      const prevIdx = seenIds.get(id);
      if (prevIdx !== undefined) {
        stepErrs.push(`Duplicate step ID "${id}" (already used by step ${prevIdx + 1}).`);
      } else {
        seenIds.set(id, idx);
      }
    }
    if (id && topo.cycleIds.includes(id)) {
      stepErrs.push(`Step "${id}" is part of a dependency cycle.`);
    }

    const stringsToCheck: string[] = [];
    if (st.for_each !== undefined) {
      if (st.kind === "checkpoint") {
        stepErrs.push("Human checkpoint steps do not support batch fan-out (for_each).");
      } else if (typeof st.for_each === "string") {
        const trimmedForEach = st.for_each.trim();
        if (!trimmedForEach) {
          stepErrs.push("Batch fan-out (for_each) cannot be empty.");
        } else {
          if (trimmedForEach.startsWith("[")) {
            try {
              const parsed = JSON.parse(trimmedForEach);
              if (!Array.isArray(parsed) || parsed.length === 0 || parsed.length > 50) {
                stepErrs.push("Batch fan-out (for_each) array must have 1 to 50 items.");
              }
            } catch {
              stepErrs.push("Batch fan-out (for_each) JSON array is invalid.");
            }
          }
          stringsToCheck.push(st.for_each);
        }
      } else if (Array.isArray(st.for_each)) {
        const arr = st.for_each as unknown[];
        if (arr.length === 0 || arr.length > 50) {
          stepErrs.push("Batch fan-out (for_each) array must have 1 to 50 items.");
        }
      }
    }
    if (typeof st.when === "string" && st.when.startsWith("expr:") && !st.when.slice(5).trim()) {
      stepErrs.push("Condition expression after expr: cannot be empty.");
    }

    if (st.kind === "skill") {
      if (!String(st.skill_id ?? "").trim()) {
        stepErrs.push("Select a built-in skill.");
      }
      for (const v of Object.values(st.args ?? {})) {
        stringsToCheck.push(String(v ?? ""));
      }
    } else if (st.kind === "agent") {
      const p = String(st.prompt ?? "");
      if (!p.trim()) {
        stepErrs.push("Agent instruction cannot be empty.");
      }
      stringsToCheck.push(p);
    } else if (st.kind === "checkpoint") {
      const p = String(st.checkpoint?.prompt ?? "");
      if (!p.trim()) {
        stepErrs.push("Checkpoint review prompt cannot be empty.");
      }
      stringsToCheck.push(p);
      if (st.checkpoint?.input_mode === "choice") {
        const choices = (st.checkpoint?.choices ?? []).map((c) => c.trim()).filter(Boolean);
        if (choices.length === 0) {
          stepErrs.push("Add at least one choice option for this checkpoint.");
        }
      }
      for (const f of st.checkpoint?.inspect_files ?? []) {
        stringsToCheck.push(String(f ?? ""));
      }
    } else if (st.kind === "export") {
      const p = String(st.export?.path ?? "");
      if (!p.trim()) {
        stepErrs.push("Export source file path cannot be empty.");
      }
      stringsToCheck.push(p);
    } else if (st.kind === "deliver") {
      const cid = String(st.deliver?.connector_id ?? "").trim();
      if (!cid) {
        stepErrs.push("Select or enter an outbound connector for delivery.");
      }
      if (st.deliver?.file) stringsToCheck.push(st.deliver.file);
      if (typeof st.deliver?.payload === "string") {
        stringsToCheck.push(st.deliver.payload);
      } else if (st.deliver?.payload && typeof st.deliver.payload === "object") {
        stringsToCheck.push(JSON.stringify(st.deliver.payload));
      }
    } else if (st.kind === "subpipeline") {
      const rid = String(st.routine_id ?? "").trim();
      if (!rid) {
        stepErrs.push("Select or enter a routine for this subpipeline step.");
      }
      for (const v of Object.values(st.inputs ?? {})) {
        stringsToCheck.push(String(v ?? ""));
      }
    }

    if (st.timeout_secs !== undefined && (!Number.isFinite(st.timeout_secs) || st.timeout_secs <= 0 || st.timeout_secs > 86400)) {
      stepErrs.push("Timeout must be between 1 and 86400 seconds.");
    }
    if (st.retry_max !== undefined && (!Number.isInteger(st.retry_max) || st.retry_max < 0 || st.retry_max > 10)) {
      stepErrs.push("Max retries must be between 0 and 10.");
    }
    if (st.retry_backoff_ms !== undefined && (!Number.isFinite(st.retry_backoff_ms) || st.retry_backoff_ms < 0 || st.retry_backoff_ms > 300000)) {
      stepErrs.push("Retry backoff must be between 0 and 300000 ms.");
    }
    if (typeof st.on_error === "string" && st.on_error.startsWith("goto:")) {
      const targetId = st.on_error.slice("goto:".length).trim();
      if (!targetId) {
        stepErrs.push("Select a target step ID for on_error goto.");
      } else if (targetId === id) {
        stepErrs.push(`Step "${id}" cannot jump to itself on error.`);
      } else if (!steps.some((other) => other.id.trim() === targetId)) {
        stepErrs.push(`on_error target step "${targetId}" does not exist.`);
      }
    }

    for (const s of stringsToCheck) {
      for (const tErr of validateTemplateText(s, idx, steps, hasDagDeps)) {
        if (!stepErrs.includes(tErr)) stepErrs.push(tErr);
      }
    }

    if (stepErrs.length > 0) {
      errorsByStep[idx] = stepErrs;
    }
  });

  const valid = globalErrors.length === 0 && Object.keys(errorsByStep).length === 0;
  return {
    valid,
    errorsByStep,
    globalErrors,
    topologicalOrder: valid ? topo.topologicalOrder : [],
    stages: valid ? topo.stages : [],
  };
}

export function makeDefaultStep(kind: PipelineStepKind, index: number, prevId?: string): PipelineStepSpec {
  const n = index + 1;
  if (kind === "skill") {
    return {
      id: `skill_${n}`,
      label: "Run built-in skill",
      kind: "skill",
      skill_id: "clean_dataset",
      args: { input_file: "{{inputs.file}}" },
      when: "always",
    };
  }
  if (kind === "agent") {
    return {
      id: `agent_${n}`,
      label: "Analyze and draft deliverable",
      kind: "agent",
      prompt: prevId
        ? `Review {{steps.${prevId}.files[0]}} and draft a client-ready summary.`
        : "Review workspace files and draft a client-ready summary.",
      when: "always",
      ...(prevId ? { depends_on: [prevId] } : {}),
    };
  }
  if (kind === "checkpoint") {
    return {
      id: `review_${n}`,
      label: "Human review checkpoint",
      kind: "checkpoint",
      checkpoint: {
        prompt: "Review the generated output before continuing.",
        input_mode: "approve_reject",
        choices: ["Proceed as-is", "Revise output", "Abort"],
        inspect_files: prevId ? [`{{steps.${prevId}.files[0]}}`] : [],
      },
      when: "always",
      ...(prevId ? { depends_on: [prevId] } : {}),
    };
  }
  if (kind === "deliver") {
    return {
      id: `deliver_${n}`,
      label: "Deliver to connector",
      kind: "deliver",
      deliver: {
        connector_id: "",
        file: prevId ? `{{steps.${prevId}.files[0]}}` : "",
        payload: prevId ? { summary: `{{steps.${prevId}.output}}` } : undefined,
      },
      when: "on_all_ok",
      ...(prevId ? { depends_on: [prevId] } : {}),
    };
  }
  if (kind === "subpipeline") {
    return {
      id: `sub_${n}`,
      label: "Run subpipeline",
      kind: "subpipeline",
      routine_id: "",
      inputs: {},
      when: "always",
      ...(prevId ? { depends_on: [prevId] } : {}),
    };
  }
  return {
    id: `export_${n}`,
    label: "Export deliverable",
    kind: "export",
    export: {
      path: prevId ? `{{steps.${prevId}.files[0]}}` : "{{inputs.file}}",
      format: "docx",
    },
    when: "on_previous_ok",
    ...(prevId ? { depends_on: [prevId] } : {}),
  };
}

function serializeDeliverPayload(payload: unknown): string {
  if (payload === undefined || payload === null) return "";
  if (typeof payload === "string") return payload;
  try {
    return JSON.stringify(payload, null, 2);
  } catch {
    return String(payload);
  }
}

function parseDeliverPayloadInput(raw: string): unknown {
  const trimmed = raw.trim();
  if (!trimmed) return undefined;
  if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
    try {
      return JSON.parse(trimmed);
    } catch {
      return raw;
    }
  }
  return raw;
}

export function PipelineBuilder({
  steps,
  onChange,
  skills,
  workspaceFiles = [],
  outboundConnectors,
  routines = [],
}: {
  steps: PipelineStepSpec[];
  onChange: (next: PipelineStepSpec[]) => void;
  skills: SkillMeta[];
  workspaceFiles?: WorkspaceFile[];
  outboundConnectors?: OutboundConnector[];
  routines?: Routine[];
}) {
  const [newArgKey, setNewArgKey] = useState<Record<number, string>>({});
  const [newSubInputKey, setNewSubInputKey] = useState<Record<number, string>>({});
  const [resilienceOpen, setResilienceOpen] = useState<Record<number, boolean>>({});
  const [fetchedConnectors, setFetchedConnectors] = useState<OutboundConnector[]>([]);
  const [connectorsErr, setConnectorsErr] = useState(false);
  const [validatingDag, setValidatingDag] = useState(false);
  const [dagResult, setDagResult] = useState<ApiPipelineValidationResult | null>(null);
  const validateLockRef = useRef(false);
  const validateSeqRef = useRef(0);
  const validatedSigRef = useRef<string | null>(null);
  const validation = useMemo(() => validatePipelineSteps(steps), [steps]);
  const stepsSig = useMemo(() => JSON.stringify(steps), [steps]);

  useEffect(() => {
    if (outboundConnectors !== undefined) {
      setConnectorsErr(false);
      return;
    }
    let active = true;
    void listConnectors()
      .then((res) => {
        if (!active) return;
        if (!res.ok) {
          setConnectorsErr(true);
          setFetchedConnectors([]);
          return;
        }
        setConnectorsErr(false);
        setFetchedConnectors(Array.isArray(res.data?.outbound) ? res.data.outbound : []);
      })
      .catch(() => {
        if (!active) return;
        setConnectorsErr(true);
        setFetchedConnectors([]);
      });
    return () => {
      active = false;
    };
  }, [outboundConnectors, steps.length]);

  useEffect(() => {
    if (validatedSigRef.current !== null && validatedSigRef.current !== stepsSig) {
      validateSeqRef.current += 1;
      validatedSigRef.current = null;
      setDagResult(null);
    }
  }, [stepsSig]);

  const connectors = useMemo(() => {
    const raw = Array.isArray(outboundConnectors) ? outboundConnectors : fetchedConnectors;
    return Array.isArray(raw)
      ? raw.filter((c): c is OutboundConnector => Boolean(c && typeof c.id === "string" && c.id.trim()))
      : [];
  }, [outboundConnectors, fetchedConnectors]);

  const commitSteps = (next: PipelineStepSpec[]) => {
    validateSeqRef.current += 1;
    validatedSigRef.current = null;
    setDagResult(null);
    onChange(next);
  };

  const updateStep = (idx: number, patch: Partial<PipelineStepSpec>) => {
    commitSteps(steps.map((s, i) => (i === idx ? { ...s, ...patch } : s)));
  };

  const changeKind = (idx: number, kind: PipelineStepKind) => {
    const cur = steps[idx];
    const prevId = idx > 0 ? steps[idx - 1].id : undefined;
    const template = makeDefaultStep(kind, idx, prevId);
    const defaultCid = kind === "deliver" && connectors[0]?.id ? connectors[0].id : undefined;
    commitSteps(
      steps.map((s, i) =>
        i === idx
          ? {
              ...template,
              ...(kind === "deliver" && defaultCid
                ? { deliver: { ...(template.deliver ?? { connector_id: "" }), connector_id: defaultCid } }
                : {}),
              id: cur.id || template.id,
              label: cur.label && cur.label !== KIND_META[cur.kind].label ? cur.label : template.label,
              when: cur.when || "always",
              depends_on: cur.depends_on,
              for_each: cur.for_each,
            }
          : s
      )
    );
  };

  const moveStep = (idx: number, dir: -1 | 1) => {
    const target = idx + dir;
    if (target < 0 || target >= steps.length) return;
    const copy = steps.slice();
    const [item] = copy.splice(idx, 1);
    copy.splice(target, 0, item);
    commitSteps(copy);
  };

  const removeStep = (idx: number) => {
    const removedId = steps[idx]?.id;
    const next = steps
      .filter((_, i) => i !== idx)
      .map((s) =>
        removedId && s.depends_on?.includes(removedId)
          ? { ...s, depends_on: s.depends_on.filter((d) => d !== removedId) }
          : s
      );
    commitSteps(next);
  };

  const addStep = (kind: PipelineStepKind) => {
    const prevId = steps.length > 0 ? steps[steps.length - 1].id : undefined;
    const created = makeDefaultStep(kind, steps.length, prevId);
    if (kind === "deliver" && created.deliver && !created.deliver.connector_id && connectors[0]?.id) {
      created.deliver.connector_id = connectors[0].id;
    }
    commitSteps([...steps, created]);
  };

  const toggleDependency = (idx: number, targetId: string) => {
    const cur = steps[idx];
    if (!cur || !targetId || targetId === cur.id) return;
    const existing = Array.isArray(cur.depends_on) ? cur.depends_on : [];
    const nextDeps = existing.includes(targetId)
      ? existing.filter((d) => d !== targetId)
      : [...existing, targetId];
    updateStep(idx, { depends_on: nextDeps.length > 0 ? nextDeps : undefined });
  };

  const insertTokenIntoStep = (idx: number, token: string) => {
    const st = steps[idx];
    if (!st) return;
    if (st.kind === "skill") {
      const args = { ...(st.args ?? {}) };
      const curInput = String(args.input_file ?? "");
      args.input_file = curInput && !curInput.includes("{{") ? token : curInput ? `${curInput} ${token}` : token;
      updateStep(idx, { args });
      return;
    }
    if (st.kind === "agent") {
      const cur = String(st.prompt ?? "");
      updateStep(idx, { prompt: cur ? `${cur} ${token}` : token });
      return;
    }
    if (st.kind === "checkpoint") {
      const cp = st.checkpoint ?? { prompt: "", input_mode: "approve_reject" as const };
      if (token.includes(".files")) {
        const list = [...(cp.inspect_files ?? [])];
        if (!list.includes(token)) list.push(token);
        updateStep(idx, { checkpoint: { ...cp, inspect_files: list } });
      } else {
        const cur = String(cp.prompt ?? "");
        updateStep(idx, { checkpoint: { ...cp, prompt: cur ? `${cur} ${token}` : token } });
      }
      return;
    }
    if (st.kind === "export") {
      const exp = st.export ?? { path: "", format: "docx" as const };
      updateStep(idx, { export: { ...exp, path: token } });
      return;
    }
    if (st.kind === "deliver") {
      const del = st.deliver ?? { connector_id: "" };
      if (token.includes(".files")) {
        updateStep(idx, { deliver: { ...del, file: token } });
      } else {
        const cur = serializeDeliverPayload(del.payload);
        updateStep(idx, { deliver: { ...del, payload: cur ? `${cur} ${token}` : token } });
      }
      return;
    }
    if (st.kind === "subpipeline") {
      const inputs = { ...(st.inputs ?? {}) };
      const firstKey = Object.keys(inputs)[0] || "file";
      const curVal = String(inputs[firstKey] ?? "");
      inputs[firstKey] = curVal && !curVal.includes("{{") ? token : curVal ? `${curVal} ${token}` : token;
      updateStep(idx, { inputs });
    }
  };

  const handleValidateDag = async () => {
    if (validateLockRef.current || validatingDag) return;
    validateLockRef.current = true;
    setValidatingDag(true);
    const reqSeq = ++validateSeqRef.current;
    const reqSig = stepsSig;
    try {
      const localCheck = validatePipelineSteps(steps);
      const apiRes = await validatePipeline(steps);
      if (reqSeq !== validateSeqRef.current) return;
      validatedSigRef.current = reqSig;
      if (apiRes.ok) {
        const srv = apiRes.data;
        const mergedErrors = Array.from(
          new Set([
            ...(Array.isArray(srv.errors) ? srv.errors : []),
            ...localCheck.globalErrors,
            ...Object.entries(localCheck.errorsByStep).flatMap(([k, msgs]) =>
              msgs.map((m) => `Step ${Number(k) + 1}: ${m}`)
            ),
          ])
        );
        const valid = Boolean(srv.valid) && localCheck.valid;
        setDagResult({
          ok: valid,
          valid,
          errors: mergedErrors,
          topological_order: valid
            ? Array.isArray(srv.topological_order) && srv.topological_order.length > 0
              ? srv.topological_order
              : localCheck.topologicalOrder
            : [],
          stages: valid
            ? Array.isArray(srv.stages) && srv.stages.length > 0
              ? srv.stages
              : localCheck.stages
            : [],
        });
      } else {
        const fallbackErrs = [
          ...localCheck.globalErrors,
          ...Object.entries(localCheck.errorsByStep).flatMap(([k, msgs]) =>
            msgs.map((m) => `Step ${Number(k) + 1}: ${m}`)
          ),
        ];
        setDagResult({
          ok: localCheck.valid,
          valid: localCheck.valid,
          errors: fallbackErrs.length > 0 ? fallbackErrs : [apiRes.error || "Pipeline validation failed."],
          topological_order: localCheck.topologicalOrder,
          stages: localCheck.stages,
        });
      }
    } finally {
      validateLockRef.current = false;
      setValidatingDag(false);
    }
  };

  return (
    <div id="pipeline-builder" className="flex min-w-0 flex-col gap-3">
      {steps.length === 0 ? (
        <div className="rounded-sm border border-dashed border-hairline bg-surface px-3 py-4 text-center text-xs text-ink-2">
          Add typed steps below to compose a resumable DAG pipeline with skills, AI agents, nested subpipelines, parallel batch fan-out, checkpoints, exports, and outbound deliveries.
        </div>
      ) : (
        <ol className="flex min-w-0 flex-col gap-2.5">
          {steps.map((st, idx) => {
            const meta = KIND_META[st.kind] ?? KIND_META.skill;
            const KindIcon = meta.icon;
            const otherSteps = steps.filter((_, oIdx) => oIdx !== idx && Boolean(steps[oIdx]?.id?.trim()));
            const varChips: string[] = ["{{inputs.file}}"];
            if (st.for_each !== undefined) {
              varChips.push("{{item}}");
            }
            for (const other of steps) {
              if (!other.id || other.id === st.id) continue;
              varChips.push(`{{steps.${other.id}.files[0]}}`);
              if (other.kind === "checkpoint") {
                varChips.push(`{{steps.${other.id}.human_answer}}`);
              } else {
                varChips.push(`{{steps.${other.id}.output}}`);
              }
            }

            const argEntries = Object.entries(st.args ?? {});
            const subInputEntries = Object.entries(st.inputs ?? {});
            const stepErrs = validation.errorsByStep[idx] ?? [];
            const selectedDeps = Array.isArray(st.depends_on) ? st.depends_on : [];
            const isExprWhen = typeof st.when === "string" && st.when.startsWith("expr:");
            const whenSelectVal = isExprWhen ? "expr" : st.when || "always";
            const forEachActive = st.for_each !== undefined;
            const hasResilience =
              st.timeout_secs !== undefined ||
              st.retry_max !== undefined ||
              st.retry_backoff_ms !== undefined ||
              (st.on_error !== undefined && st.on_error !== "fail");
            const showResilience = Boolean(resilienceOpen[idx] || hasResilience);
            const onErrorMode =
              typeof st.on_error === "string" && st.on_error.startsWith("goto:")
                ? "goto"
                : st.on_error || "fail";
            const gotoTarget =
              typeof st.on_error === "string" && st.on_error.startsWith("goto:")
                ? st.on_error.slice("goto:".length)
                : "";

            return (
              <li
                key={`${idx}-${st.id}`}
                data-pipeline-step-index={idx}
                data-pipeline-step-kind={st.kind}
                className={cn(
                  "flex min-w-0 flex-col gap-2.5 rounded-card border bg-surface p-3",
                  stepErrs.length > 0 ? "border-dot-red" : "border-hairline"
                )}
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="flex min-w-0 flex-wrap items-center gap-1.5">
                    <span className="mono inline-flex size-5 shrink-0 items-center justify-center rounded-full bg-board text-xs font-medium text-ink">
                      {idx + 1}
                    </span>
                    <KindIcon className="size-3.5 shrink-0 text-icon" strokeWidth={1.5} aria-hidden />
                    <select
                      aria-label={`Step ${idx + 1} type`}
                      value={st.kind}
                      onChange={(e) => changeKind(idx, e.target.value as PipelineStepKind)}
                      className="focus-ring h-8 min-w-0 max-w-full rounded-sm border border-input bg-board/60 px-2 text-xs font-medium text-ink outline-none max-md:min-h-11"
                    >
                      <option value="skill">Skill</option>
                      <option value="agent">AI Agent</option>
                      <option value="checkpoint">Human Checkpoint</option>
                      <option value="export">Export</option>
                      <option value="deliver">Deliver</option>
                      <option value="subpipeline">Subpipeline</option>
                    </select>
                    <Badge variant="outline" size="sm" className="text-xs text-ink-2">
                      {meta.badge}
                    </Badge>
                  </div>

                  <div className="flex shrink-0 items-center gap-1">
                    <Button
                      type="button"
                      size="icon-xs"
                      variant="ghost"
                      disabled={idx === 0}
                      aria-label={`Move step ${idx + 1} up`}
                      className="max-md:min-h-11 max-md:min-w-11"
                      onClick={() => moveStep(idx, -1)}
                    >
                      <ArrowUp className="size-3.5" strokeWidth={1.5} aria-hidden />
                    </Button>
                    <Button
                      type="button"
                      size="icon-xs"
                      variant="ghost"
                      disabled={idx === steps.length - 1}
                      aria-label={`Move step ${idx + 1} down`}
                      className="max-md:min-h-11 max-md:min-w-11"
                      onClick={() => moveStep(idx, 1)}
                    >
                      <ArrowDown className="size-3.5" strokeWidth={1.5} aria-hidden />
                    </Button>
                    <Button
                      type="button"
                      size="icon-xs"
                      variant="ghost"
                      aria-label={`Remove step ${idx + 1}`}
                      className="max-md:min-h-11 max-md:min-w-11 text-ink-2 hover:text-ink"
                      onClick={() => removeStep(idx)}
                    >
                      <Trash2 className="size-3.5" strokeWidth={1.5} aria-hidden />
                    </Button>
                  </div>
                </div>

                <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
                  <div className="flex min-w-0 flex-col gap-1">
                    <label className="text-xs font-medium text-ink-2">Step ID</label>
                    <Input
                      value={st.id}
                      aria-label={`Step ${idx + 1} ID`}
                      placeholder={`step_${idx + 1}`}
                      onChange={(e) => updateStep(idx, { id: sanitizeStepId(e.target.value, `step_${idx + 1}`) })}
                      className="h-8 w-full min-w-0 text-xs max-md:min-h-11"
                    />
                  </div>
                  <div className="flex min-w-0 flex-col gap-1">
                    <label className="text-xs font-medium text-ink-2">Step label</label>
                    <Input
                      value={st.label}
                      aria-label={`Step ${idx + 1} label`}
                      placeholder="Short plain label"
                      onChange={(e) => updateStep(idx, { label: e.target.value })}
                      className="h-8 w-full min-w-0 text-xs max-md:min-h-11"
                    />
                  </div>
                  <div className="flex min-w-0 flex-col gap-1">
                    <label className="text-xs font-medium text-ink-2">Run condition</label>
                    <select
                      aria-label={`Step ${idx + 1} condition`}
                      value={whenSelectVal}
                      onChange={(e) => {
                        const v = e.target.value;
                        updateStep(idx, {
                          when: v === "expr" ? (isExprWhen ? st.when : "expr:steps.clean.metrics.anomalies == 0") : (v as PipelineWhen),
                        });
                      }}
                      className="focus-ring h-8 w-full min-w-0 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                    >
                      <option value="always">Always run</option>
                      <option value="on_previous_ok">Only if previous succeeded</option>
                      <option value="on_all_ok">Only if all dependencies succeeded</option>
                      <option value="on_anomaly">Only if anomaly flagged</option>
                      <option value="expr">Custom expression (expr:...)</option>
                    </select>
                  </div>
                </div>

                {isExprWhen && (
                  <div className="flex flex-col gap-1">
                    <label className="text-xs font-medium text-ink-2">Condition expression</label>
                    <Input
                      value={st.when ?? "expr:"}
                      aria-label={`Step ${idx + 1} expression`}
                      placeholder="expr:steps.clean.metrics.anomalies == 0"
                      onChange={(e) => {
                        const val = e.target.value;
                        updateStep(idx, { when: val.startsWith("expr:") ? val : `expr:${val}` });
                      }}
                      className="h-8 text-xs max-md:min-h-11"
                    />
                  </div>
                )}

                {otherSteps.length > 0 && (
                  <div className="flex flex-wrap items-center gap-1.5 border-t border-hairline pt-2">
                    <span className="flex items-center gap-1 text-xs font-medium text-ink-2">
                      <GitBranch className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                      Depends on:
                    </span>
                    {otherSteps.map((dep) => {
                      const active = selectedDeps.includes(dep.id);
                      return (
                        <button
                          key={dep.id}
                          type="button"
                          aria-pressed={active}
                          aria-label={`Step ${idx + 1} depends on ${dep.id}`}
                          data-dep-chip={`${idx}:${dep.id}`}
                          onClick={() => toggleDependency(idx, dep.id)}
                          className={cn(
                            "rounded-chip border px-2 py-0.5 text-xs transition-[color,background-color,border-color] max-md:min-h-11",
                            active
                              ? "border-blue bg-blue-soft font-medium text-ink"
                              : "border-hairline bg-board/60 text-ink-2 hover:bg-board hover:text-ink"
                          )}
                        >
                          {dep.id}
                        </button>
                      );
                    })}
                  </div>
                )}

                {st.kind !== "checkpoint" && (
                  <div className="flex min-w-0 flex-col gap-1.5 border-t border-hairline pt-2">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <div className="flex flex-wrap items-center gap-1.5">
                        <button
                          type="button"
                          aria-pressed={forEachActive}
                          aria-label={`Step ${idx + 1} batch fan-out`}
                          data-toggle-for-each={idx}
                          onClick={() => {
                            if (forEachActive) {
                              updateStep(idx, { for_each: undefined });
                            } else {
                              const prev = idx > 0 ? steps[idx - 1]?.id : undefined;
                              updateStep(idx, { for_each: prev ? `{{steps.${prev}.files}}` : "{{inputs.items}}" });
                            }
                          }}
                          className={cn(
                            "inline-flex items-center gap-1.5 rounded-chip border px-2.5 py-0.5 text-xs transition-[color,background-color,border-color] max-md:min-h-11",
                            forEachActive
                              ? "border-blue bg-blue-soft font-medium text-ink"
                              : "border-hairline bg-board/60 text-ink-2 hover:bg-board hover:text-ink"
                          )}
                        >
                          Batch fan-out (for_each)
                        </button>
                        <button
                          type="button"
                          aria-pressed={showResilience}
                          aria-label={`Step ${idx + 1} retry and error policy`}
                          data-toggle-resilience={idx}
                          onClick={() =>
                            setResilienceOpen((prev) => ({ ...prev, [idx]: !showResilience }))
                          }
                          className={cn(
                            "inline-flex items-center gap-1.5 rounded-chip border px-2.5 py-0.5 text-xs transition-[color,background-color,border-color] max-md:min-h-11",
                            hasResilience || showResilience
                              ? "border-blue bg-blue-soft font-medium text-ink"
                              : "border-hairline bg-board/60 text-ink-2 hover:bg-board hover:text-ink"
                          )}
                        >
                          Retry & timeout
                        </button>
                      </div>
                      {forEachActive && (
                        <span className="text-xs text-ink-3">
                          Runs once per item via {`{{item}}`} (max 50).
                        </span>
                      )}
                    </div>
                    {forEachActive && (
                      <Input
                        value={st.for_each ?? ""}
                        aria-label={`Step ${idx + 1} for_each template`}
                        placeholder="{{steps.prev.files}} or JSON array / newline list"
                        onChange={(e) => updateStep(idx, { for_each: e.target.value })}
                        className="h-8 w-full min-w-0 text-xs max-md:min-h-11"
                      />
                    )}
                    {showResilience && (
                      <div
                        data-step-resilience={idx}
                        className="grid grid-cols-1 gap-2 rounded-sm border border-hairline bg-board/40 p-2.5 sm:grid-cols-2 lg:grid-cols-4"
                      >
                        <div className="flex min-w-0 flex-col gap-1">
                          <label className="text-xs font-medium text-ink-2">Timeout (secs)</label>
                          <Input
                            type="number"
                            min={1}
                            max={86400}
                            value={st.timeout_secs ?? ""}
                            aria-label={`Step ${idx + 1} timeout secs`}
                            data-step-timeout={idx}
                            placeholder="e.g. 60"
                            onChange={(e) => {
                              const raw = e.target.value.trim();
                              updateStep(idx, { timeout_secs: raw ? Number(raw) : undefined });
                            }}
                            className="h-8 w-full min-w-0 text-xs max-md:min-h-11"
                          />
                        </div>
                        <div className="flex min-w-0 flex-col gap-1">
                          <label className="text-xs font-medium text-ink-2">Max retries</label>
                          <Input
                            type="number"
                            min={0}
                            max={10}
                            value={st.retry_max ?? ""}
                            aria-label={`Step ${idx + 1} max retries`}
                            data-step-retry-max={idx}
                            placeholder="0"
                            onChange={(e) => {
                              const raw = e.target.value.trim();
                              updateStep(idx, { retry_max: raw ? Math.floor(Number(raw)) : undefined });
                            }}
                            className="h-8 w-full min-w-0 text-xs max-md:min-h-11"
                          />
                        </div>
                        <div className="flex min-w-0 flex-col gap-1">
                          <label className="text-xs font-medium text-ink-2">Backoff (ms)</label>
                          <Input
                            type="number"
                            min={0}
                            max={300000}
                            value={st.retry_backoff_ms ?? ""}
                            aria-label={`Step ${idx + 1} retry backoff ms`}
                            data-step-retry-backoff={idx}
                            placeholder="500"
                            onChange={(e) => {
                              const raw = e.target.value.trim();
                              updateStep(idx, { retry_backoff_ms: raw ? Math.floor(Number(raw)) : undefined });
                            }}
                            className="h-8 w-full min-w-0 text-xs max-md:min-h-11"
                          />
                        </div>
                        <div className="flex min-w-0 flex-col gap-1">
                          <label className="text-xs font-medium text-ink-2">On error</label>
                          <select
                            aria-label={`Step ${idx + 1} on_error`}
                            data-step-on-error={idx}
                            value={onErrorMode}
                            onChange={(e) => {
                              const mode = e.target.value;
                              if (mode === "goto") {
                                const fallbackTarget = otherSteps[0]?.id ?? "";
                                updateStep(idx, {
                                  on_error: fallbackTarget ? (`goto:${fallbackTarget}` as PipelineOnError) : "goto:",
                                });
                              } else {
                                updateStep(idx, { on_error: mode as PipelineOnError });
                              }
                            }}
                            className="focus-ring h-8 w-full min-w-0 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                          >
                            <option value="fail">Fail pipeline</option>
                            <option value="continue">Continue next step</option>
                            <option value="goto">Jump to step (goto)</option>
                          </select>
                        </div>
                        {onErrorMode === "goto" && (
                          <div className="flex min-w-0 flex-col gap-1 sm:col-span-2 lg:col-span-4">
                            <label className="text-xs font-medium text-ink-2">Jump to step on error</label>
                            {otherSteps.length > 0 ? (
                              <select
                                aria-label={`Step ${idx + 1} on_error goto target`}
                                data-step-on-error-goto={idx}
                                value={gotoTarget}
                                onChange={(e) =>
                                  updateStep(idx, { on_error: `goto:${e.target.value}` as PipelineOnError })
                                }
                                className="focus-ring h-8 w-full min-w-0 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                              >
                                <option value="">Select fallback step…</option>
                                {otherSteps.map((o) => (
                                  <option key={o.id} value={o.id}>
                                    {o.label ? `${o.id} (${o.label})` : o.id}
                                  </option>
                                ))}
                              </select>
                            ) : (
                              <Input
                                value={gotoTarget}
                                aria-label={`Step ${idx + 1} on_error goto target`}
                                data-step-on-error-goto={idx}
                                placeholder="target_step_id"
                                onChange={(e) =>
                                  updateStep(idx, { on_error: `goto:${e.target.value.trim()}` as PipelineOnError })
                                }
                                className="h-8 w-full min-w-0 text-xs max-md:min-h-11"
                              />
                            )}
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                )}

                {st.kind === "skill" && (
                  <div className="flex min-w-0 flex-col gap-2 rounded-sm border border-hairline bg-board/40 p-2.5">
                    <div className="flex min-w-0 flex-col gap-1">
                      <label className="text-xs font-medium text-ink">Built-in skill</label>
                      <select
                        aria-label={`Step ${idx + 1} skill`}
                        value={st.skill_id ?? ""}
                        onChange={(e) => updateStep(idx, { skill_id: e.target.value })}
                        className="focus-ring h-8 w-full min-w-0 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                      >
                        {skills.map((sk) => (
                          <option key={sk.id} value={sk.id}>
                            {sk.title} ({sk.id})
                          </option>
                        ))}
                      </select>
                    </div>

                    <div className="flex min-w-0 flex-col gap-1.5">
                      <span className="text-xs font-medium text-ink-2">Arguments</span>
                      {argEntries.map(([k, v]) => (
                        <div key={k} className="flex min-w-0 flex-wrap items-center gap-1.5">
                          <span className="min-w-[96px] text-xs font-medium text-ink-2 [overflow-wrap:anywhere]">{k}</span>
                          <Input
                            value={String(v ?? "")}
                            aria-label={`Step ${idx + 1} arg ${k}`}
                            placeholder="Value or {{steps.prev.files[0]}}"
                            onChange={(e) =>
                              updateStep(idx, { args: { ...(st.args ?? {}), [k]: e.target.value } })
                            }
                            className="h-8 min-w-[120px] flex-1 text-xs max-md:min-h-11"
                          />
                          {workspaceFiles.length > 0 && k.includes("file") && (
                            <select
                              aria-label={`Pick workspace file for ${k}`}
                              value=""
                              onChange={(e) => {
                                if (!e.target.value) return;
                                updateStep(idx, { args: { ...(st.args ?? {}), [k]: e.target.value } });
                              }}
                              className="focus-ring h-8 min-w-0 max-w-full rounded-sm border border-input bg-surface px-2 text-xs text-ink-2 outline-none max-md:min-h-11"
                            >
                              <option value="">Workspace file…</option>
                              {workspaceFiles.slice(0, 20).map((f) => (
                                <option key={f.path} value={f.path}>
                                  {f.path}
                                </option>
                              ))}
                            </select>
                          )}
                          <Button
                            type="button"
                            size="xs"
                            variant="ghost"
                            aria-label={`Remove argument ${k}`}
                            onClick={() => {
                              const nextArgs = { ...(st.args ?? {}) };
                              delete nextArgs[k];
                              updateStep(idx, { args: nextArgs });
                            }}
                          >
                            Remove
                          </Button>
                        </div>
                      ))}
                      <div className="flex min-w-0 flex-wrap items-center gap-1.5 pt-0.5">
                        <Input
                          value={newArgKey[idx] ?? ""}
                          placeholder="Add arg name (e.g. output_title)"
                          aria-label={`New arg key for step ${idx + 1}`}
                          onChange={(e) => setNewArgKey((prev) => ({ ...prev, [idx]: e.target.value }))}
                          className="h-8 w-48 max-w-full min-w-0 text-xs max-md:min-h-11"
                        />
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          className="max-md:min-h-11"
                          onClick={() => {
                            const key = (newArgKey[idx] ?? "").trim().replace(/\s+/g, "_");
                            if (!key) return;
                            updateStep(idx, { args: { ...(st.args ?? {}), [key]: "" } });
                            setNewArgKey((prev) => ({ ...prev, [idx]: "" }));
                          }}
                        >
                          Add arg
                        </Button>
                      </div>
                    </div>
                  </div>
                )}

                {st.kind === "agent" && (
                  <div className="flex min-w-0 flex-col gap-1.5 rounded-sm border border-hairline bg-board/40 p-2.5">
                    <label className="text-xs font-medium text-ink">Agent instruction</label>
                    <Textarea
                      rows={2}
                      value={st.prompt ?? ""}
                      aria-label={`Step ${idx + 1} agent prompt`}
                      placeholder="What should the AI agent produce using prior outputs?"
                      onChange={(e) => updateStep(idx, { prompt: e.target.value })}
                      className="w-full min-w-0 text-xs"
                    />
                  </div>
                )}

                {st.kind === "checkpoint" && (
                  <div className="flex min-w-0 flex-col gap-2 rounded-sm border border-hairline bg-blue-soft/45 p-2.5">
                    <div className="flex min-w-0 flex-col gap-1">
                      <label className="text-xs font-medium text-ink">Question or review prompt</label>
                      <Input
                        value={st.checkpoint?.prompt ?? ""}
                        aria-label={`Step ${idx + 1} checkpoint prompt`}
                        placeholder="Review flagged items before final export"
                        onChange={(e) =>
                          updateStep(idx, {
                            checkpoint: {
                              prompt: e.target.value,
                              input_mode: st.checkpoint?.input_mode ?? "approve_reject",
                              choices: st.checkpoint?.choices,
                              inspect_files: st.checkpoint?.inspect_files,
                            },
                          })
                        }
                        className="w-full min-w-0 text-xs max-md:min-h-11"
                      />
                    </div>
                    <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                      <div className="flex min-w-0 flex-col gap-1">
                        <label className="text-xs font-medium text-ink-2">Input mode</label>
                        <select
                          aria-label={`Step ${idx + 1} checkpoint mode`}
                          value={st.checkpoint?.input_mode ?? "approve_reject"}
                          onChange={(e) =>
                            updateStep(idx, {
                              checkpoint: {
                                prompt: st.checkpoint?.prompt ?? "",
                                input_mode: e.target.value as PipelineCheckpointInputMode,
                                choices: st.checkpoint?.choices ?? ["Proceed as-is", "Revise", "Abort"],
                                inspect_files: st.checkpoint?.inspect_files,
                              },
                            })
                          }
                          className="focus-ring h-8 w-full min-w-0 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                        >
                          <option value="approve_reject">Approve or reject</option>
                          <option value="choice">Structured choices</option>
                          <option value="text">Free-text answer</option>
                        </select>
                      </div>
                      <div className="flex min-w-0 flex-col gap-1">
                        <label className="text-xs font-medium text-ink-2">Files to inspect (comma-separated)</label>
                        <Input
                          value={(st.checkpoint?.inspect_files ?? []).join(", ")}
                          aria-label={`Step ${idx + 1} inspect files`}
                          placeholder="{{steps.clean.files[0]}}"
                          onChange={(e) =>
                            updateStep(idx, {
                              checkpoint: {
                                prompt: st.checkpoint?.prompt ?? "",
                                input_mode: st.checkpoint?.input_mode ?? "approve_reject",
                                choices: st.checkpoint?.choices,
                                inspect_files: e.target.value
                                  .split(",")
                                  .map((x) => x.trim())
                                  .filter(Boolean),
                              },
                            })
                          }
                          className="h-8 w-full min-w-0 text-xs max-md:min-h-11"
                        />
                      </div>
                    </div>
                    {st.checkpoint?.input_mode === "choice" && (
                      <div className="flex flex-col gap-1">
                        <label className="text-xs font-medium text-ink-2">Choices (comma-separated)</label>
                        <Input
                          value={(st.checkpoint?.choices ?? []).join(", ")}
                          aria-label={`Step ${idx + 1} choices`}
                          placeholder="Proceed as-is, Exclude flagged rows, Abort"
                          onChange={(e) =>
                            updateStep(idx, {
                              checkpoint: {
                                prompt: st.checkpoint?.prompt ?? "",
                                input_mode: "choice",
                                choices: e.target.value
                                  .split(",")
                                  .map((x) => x.trim())
                                  .filter(Boolean),
                                inspect_files: st.checkpoint?.inspect_files,
                              },
                            })
                          }
                          className="text-xs max-md:min-h-11"
                        />
                      </div>
                    )}
                  </div>
                )}

                {st.kind === "export" && (
                  <div className="grid min-w-0 grid-cols-1 gap-2 rounded-sm border border-hairline bg-board/40 p-2.5 sm:grid-cols-[minmax(0,1fr)_140px]">
                    <div className="flex min-w-0 flex-col gap-1">
                      <label className="text-xs font-medium text-ink">Source file path</label>
                      <Input
                        value={st.export?.path ?? ""}
                        aria-label={`Step ${idx + 1} export path`}
                        placeholder="{{steps.brief.files[0]}}"
                        onChange={(e) =>
                          updateStep(idx, {
                            export: {
                              path: e.target.value,
                              format: st.export?.format ?? "docx",
                            },
                          })
                        }
                        className="h-8 w-full min-w-0 text-xs max-md:min-h-11"
                      />
                    </div>
                    <div className="flex min-w-0 flex-col gap-1">
                      <label className="text-xs font-medium text-ink">Format</label>
                      <select
                        aria-label={`Step ${idx + 1} export format`}
                        value={st.export?.format ?? "docx"}
                        onChange={(e) =>
                          updateStep(idx, {
                            export: {
                              path: st.export?.path ?? "",
                              format: e.target.value as "docx" | "xlsx",
                            },
                          })
                        }
                        className="focus-ring h-8 w-full min-w-0 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                      >
                        <option value="docx">Word (.docx)</option>
                        <option value="xlsx">Excel (.xlsx)</option>
                      </select>
                    </div>
                  </div>
                )}

                {st.kind === "deliver" && (
                  <div className="flex min-w-0 flex-col gap-2 rounded-sm border border-hairline bg-board/40 p-2.5">
                    <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                      <div className="flex min-w-0 flex-col gap-1">
                        <label className="text-xs font-medium text-ink">Outbound connector</label>
                        {connectors.length > 0 ? (
                          <select
                            aria-label={`Step ${idx + 1} connector`}
                            value={st.deliver?.connector_id ?? ""}
                            onChange={(e) =>
                              updateStep(idx, {
                                deliver: {
                                  connector_id: e.target.value,
                                  file: st.deliver?.file,
                                  payload: st.deliver?.payload,
                                },
                              })
                            }
                            className="focus-ring h-8 w-full min-w-0 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                          >
                            <option value="">Select connector…</option>
                            {connectors.map((c) => (
                              <option key={c.id} value={c.id}>
                                {c.name} ({c.kind})
                              </option>
                            ))}
                            {st.deliver?.connector_id &&
                              !connectors.some((c) => c.id === st.deliver?.connector_id) && (
                                <option value={st.deliver.connector_id}>
                                  {st.deliver.connector_id}
                                </option>
                              )}
                          </select>
                        ) : (
                          <>
                            <Input
                              value={st.deliver?.connector_id ?? ""}
                              aria-label={`Step ${idx + 1} connector`}
                              placeholder="Outbound connector ID (create in Connectors)"
                              onChange={(e) =>
                                updateStep(idx, {
                                  deliver: {
                                    connector_id: e.target.value,
                                    file: st.deliver?.file,
                                    payload: st.deliver?.payload,
                                  },
                                })
                              }
                              className="h-8 text-xs max-md:min-h-11"
                            />
                            <span
                              data-missing-connectors-hint="true"
                              className="text-xs text-ink-3"
                            >
                              {connectorsErr
                                ? "Could not load outbound connectors. Enter an ID manually or retry."
                                : "No outbound connectors configured yet. Create one in Connectors or enter an ID."}
                            </span>
                          </>
                        )}
                      </div>
                      <div className="flex min-w-0 flex-col gap-1">
                        <label className="text-xs font-medium text-ink-2">Deliver file (optional)</label>
                        <Input
                          value={st.deliver?.file ?? ""}
                          aria-label={`Step ${idx + 1} deliver file`}
                          placeholder="{{steps.prev.files[0]}}"
                          onChange={(e) =>
                            updateStep(idx, {
                              deliver: {
                                connector_id: st.deliver?.connector_id ?? "",
                                file: e.target.value || undefined,
                                payload: st.deliver?.payload,
                              },
                            })
                          }
                          className="h-8 w-full min-w-0 text-xs max-md:min-h-11"
                        />
                      </div>
                    </div>
                    <div className="flex min-w-0 flex-col gap-1">
                      <label className="text-xs font-medium text-ink-2">Payload JSON or template (optional)</label>
                      <Textarea
                        rows={2}
                        value={serializeDeliverPayload(st.deliver?.payload)}
                        aria-label={`Step ${idx + 1} deliver payload`}
                        placeholder='{"summary": "{{steps.prev.output}}"}'
                        onChange={(e) =>
                          updateStep(idx, {
                            deliver: {
                              connector_id: st.deliver?.connector_id ?? "",
                              file: st.deliver?.file,
                              payload: parseDeliverPayloadInput(e.target.value),
                            },
                          })
                        }
                        className="w-full min-w-0 text-xs"
                      />
                    </div>
                  </div>
                )}

                {st.kind === "subpipeline" && (
                  <div className="flex min-w-0 flex-col gap-2 rounded-sm border border-hairline bg-board/40 p-2.5">
                    <div className="flex min-w-0 flex-col gap-1">
                      <label className="text-xs font-medium text-ink">Target routine (subpipeline)</label>
                      {routines.length > 0 ? (
                        <select
                          aria-label={`Step ${idx + 1} subpipeline routine`}
                          data-subpipeline-routine={idx}
                          value={st.routine_id ?? ""}
                          onChange={(e) => updateStep(idx, { routine_id: e.target.value })}
                          className="focus-ring h-8 w-full min-w-0 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                        >
                          <option value="">Select routine…</option>
                          {routines.map((rt) => (
                            <option key={rt.id} value={rt.id}>
                              {rt.name} ({rt.schedule})
                            </option>
                          ))}
                          {st.routine_id && !routines.some((rt) => rt.id === st.routine_id) && (
                            <option value={st.routine_id}>{st.routine_id}</option>
                          )}
                        </select>
                      ) : (
                        <Input
                          value={st.routine_id ?? ""}
                          aria-label={`Step ${idx + 1} subpipeline routine`}
                          data-subpipeline-routine={idx}
                          placeholder="Routine ID to invoke as subpipeline"
                          onChange={(e) => updateStep(idx, { routine_id: e.target.value })}
                          className="h-8 w-full min-w-0 text-xs max-md:min-h-11"
                        />
                      )}
                    </div>

                    <div className="flex min-w-0 flex-col gap-1.5">
                      <span className="text-xs font-medium text-ink-2">Subpipeline inputs (optional)</span>
                      {subInputEntries.map(([k, v]) => (
                        <div key={k} className="flex min-w-0 flex-wrap items-center gap-1.5">
                          <span className="min-w-[80px] text-xs font-medium text-ink-2 [overflow-wrap:anywhere]">{k}</span>
                          <Input
                            value={String(v ?? "")}
                            aria-label={`Step ${idx + 1} subpipeline input ${k}`}
                            placeholder="{{steps.prev.output}}"
                            onChange={(e) =>
                              updateStep(idx, { inputs: { ...(st.inputs ?? {}), [k]: e.target.value } })
                            }
                            className="h-8 min-w-[120px] flex-1 text-xs max-md:min-h-11"
                          />
                          <Button
                            type="button"
                            size="xs"
                            variant="ghost"
                            aria-label={`Remove subpipeline input ${k}`}
                            onClick={() => {
                              const nextInputs = { ...(st.inputs ?? {}) };
                              delete nextInputs[k];
                              updateStep(idx, { inputs: nextInputs });
                            }}
                          >
                            Remove
                          </Button>
                        </div>
                      ))}
                      <div className="flex min-w-0 flex-wrap items-center gap-1.5 pt-0.5">
                        <Input
                          value={newSubInputKey[idx] ?? ""}
                          placeholder="Add input key (e.g. file)"
                          aria-label={`New input key for subpipeline step ${idx + 1}`}
                          onChange={(e) => setNewSubInputKey((prev) => ({ ...prev, [idx]: e.target.value }))}
                          className="h-8 min-w-0 w-44 max-w-full text-xs max-md:min-h-11"
                        />
                        <Button
                          type="button"
                          size="xs"
                          variant="outline"
                          className="max-md:min-h-11"
                          onClick={() => {
                            const key = (newSubInputKey[idx] ?? "").trim().replace(/\s+/g, "_");
                            if (!key) return;
                            updateStep(idx, { inputs: { ...(st.inputs ?? {}), [key]: "" } });
                            setNewSubInputKey((prev) => ({ ...prev, [idx]: "" }));
                          }}
                        >
                          Add input
                        </Button>
                      </div>
                    </div>
                  </div>
                )}

                {stepErrs.length > 0 && (
                  <div
                    role="alert"
                    data-step-validation-error={idx}
                    className="flex items-start gap-1.5 rounded-sm border border-hairline bg-status-stopped-bg px-2.5 py-1.5 text-xs text-status-stopped-fg"
                  >
                    <AlertTriangle className="mt-0.5 size-3.5 shrink-0" strokeWidth={1.5} aria-hidden />
                    <ul className="flex flex-col gap-0.5">
                      {stepErrs.map((msg, eIdx) => (
                        <li key={eIdx}>{msg}</li>
                      ))}
                    </ul>
                  </div>
                )}

                <div className="flex flex-wrap items-center gap-1.5">
                  <span className="text-xs text-ink-3">Insert variable:</span>
                  {varChips.map((chip) => (
                    <button
                      key={chip}
                      type="button"
                      onClick={() => insertTokenIntoStep(idx, chip)}
                      className="rounded-chip border border-hairline bg-board/60 px-2 py-0.5 text-xs text-ink-2 transition-[color,background-color,border-color] hover:border-ink-3 hover:bg-board hover:text-ink"
                      title={`Insert ${chip}`}
                    >
                      {chip}
                    </button>
                  ))}
                </div>
              </li>
            );
          })}
        </ol>
      )}

      {dagResult && dagResult.valid && (
        <div
          role="status"
          data-dag-validation-result="valid"
          className="flex flex-wrap items-center gap-2 rounded-sm border border-hairline bg-status-running-bg/60 px-3 py-2 text-xs text-ink"
        >
          <CheckCircle2 className="size-3.5 shrink-0 text-dot-green" strokeWidth={1.75} aria-hidden />
          <span className="font-medium">DAG valid</span>
          {dagResult.stages && dagResult.stages.length > 0 ? (
            <span className="text-ink-2 [overflow-wrap:anywhere]">
              Stages:{" "}
              {dagResult.stages
                .map((stage, sIdx) => `Stage ${sIdx + 1}: ${stage.join(" + ")}`)
                .join(" -> ")}
            </span>
          ) : (
            <span className="text-ink-2 [overflow-wrap:anywhere]">
              Order: {dagResult.topological_order.join(" -> ")}
            </span>
          )}
        </div>
      )}

      {dagResult && !dagResult.valid && (
        <div
          role="alert"
          data-dag-validation-error="true"
          className="flex items-start gap-1.5 rounded-sm border border-hairline bg-status-stopped-bg px-3 py-2 text-xs text-status-stopped-fg"
        >
          <AlertTriangle className="mt-0.5 size-3.5 shrink-0" strokeWidth={1.5} aria-hidden />
          <ul className="flex flex-col gap-0.5">
            {dagResult.errors.map((msg, eIdx) => (
              <li key={eIdx}>{msg}</li>
            ))}
          </ul>
        </div>
      )}

      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-1.5">
          <Button
            type="button"
            size="xs"
            variant="outline"
            data-add-pipeline-step="skill"
            className="max-md:min-h-11 gap-1"
            onClick={() => addStep("skill")}
          >
            <Plus className="size-3" strokeWidth={1.5} aria-hidden />
            Skill
          </Button>
          <Button
            type="button"
            size="xs"
            variant="outline"
            data-add-pipeline-step="agent"
            className="max-md:min-h-11 gap-1"
            onClick={() => addStep("agent")}
          >
            <Plus className="size-3" strokeWidth={1.5} aria-hidden />
            AI Agent
          </Button>
          <Button
            type="button"
            size="xs"
            variant="outline"
            data-add-pipeline-step="checkpoint"
            className="max-md:min-h-11 gap-1"
            onClick={() => addStep("checkpoint")}
          >
            <Plus className="size-3" strokeWidth={1.5} aria-hidden />
            Human Checkpoint
          </Button>
          <Button
            type="button"
            size="xs"
            variant="outline"
            data-add-pipeline-step="export"
            className="max-md:min-h-11 gap-1"
            onClick={() => addStep("export")}
          >
            <Plus className="size-3" strokeWidth={1.5} aria-hidden />
            Export
          </Button>
          <Button
            type="button"
            size="xs"
            variant="outline"
            data-add-pipeline-step="deliver"
            className="max-md:min-h-11 gap-1"
            onClick={() => addStep("deliver")}
          >
            <Plus className="size-3" strokeWidth={1.5} aria-hidden />
            Deliver
          </Button>
          <Button
            type="button"
            size="xs"
            variant="outline"
            data-add-pipeline-step="subpipeline"
            className="max-md:min-h-11 gap-1"
            onClick={() => addStep("subpipeline")}
          >
            <Plus className="size-3" strokeWidth={1.5} aria-hidden />
            Subpipeline
          </Button>
        </div>

        <Button
          type="button"
          size="xs"
          variant="outline"
          data-action="validate-dag"
          disabled={validatingDag || steps.length === 0}
          className="max-md:min-h-11 gap-1"
          onClick={() => void handleValidateDag()}
        >
          <GitBranch className="size-3" strokeWidth={1.5} aria-hidden />
          {validatingDag ? "Validating…" : "Validate DAG"}
        </Button>
      </div>
    </div>
  );
}
