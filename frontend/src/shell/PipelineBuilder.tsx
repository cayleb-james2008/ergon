import { useMemo, useState } from "react";
import {
  AlertTriangle,
  ArrowDown,
  ArrowUp,
  Bot,
  FileDown,
  PauseCircle,
  Plus,
  Trash2,
  Wrench,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import type {
  PipelineCheckpointInputMode,
  PipelineStepKind,
  PipelineStepSpec,
  PipelineWhen,
  WorkspaceFile,
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
};

function validateTemplateText(
  raw: string,
  stepIndex: number,
  steps: PipelineStepSpec[]
): string[] {
  const errs: string[] = [];
  const text = String(raw || "");
  if (!text.includes("{{") && !text.includes("}}")) return errs;

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
        } else if (targetIdx > stepIndex) {
          errs.push(`Step cannot reference later step "${refId}" in "{{${expr}}}".`);
        }
      }
    } else {
      errs.push(`Unsupported template "{{${expr}}}". Use {{inputs.*}} or {{steps.<prior_step>.*}}.`);
    }
    cursor = close + 2;
  }
  return errs;
}

/** Validate typed pipeline steps for empty prompts, duplicate IDs, and broken or circular templates. */
export function validatePipelineSteps(steps: PipelineStepSpec[]): PipelineValidationResult {
  const errorsByStep: Record<number, string[]> = {};
  const globalErrors: string[] = [];
  if (steps.length === 0) {
    globalErrors.push("Add at least one pipeline step.");
    return { valid: false, errorsByStep, globalErrors };
  }

  const seenIds = new Map<string, number>();
  steps.forEach((st, idx) => {
    const stepErrs: string[] = [];
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

    const stringsToCheck: string[] = [];
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
    }

    for (const s of stringsToCheck) {
      for (const tErr of validateTemplateText(s, idx, steps)) {
        if (!stepErrs.includes(tErr)) stepErrs.push(tErr);
      }
    }

    if (stepErrs.length > 0) {
      errorsByStep[idx] = stepErrs;
    }
  });

  const valid = globalErrors.length === 0 && Object.keys(errorsByStep).length === 0;
  return { valid, errorsByStep, globalErrors };
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
  };
}

export function PipelineBuilder({
  steps,
  onChange,
  skills,
  workspaceFiles = [],
}: {
  steps: PipelineStepSpec[];
  onChange: (next: PipelineStepSpec[]) => void;
  skills: SkillMeta[];
  workspaceFiles?: WorkspaceFile[];
}) {
  const [newArgKey, setNewArgKey] = useState<Record<number, string>>({});
  const validation = useMemo(() => validatePipelineSteps(steps), [steps]);

  const updateStep = (idx: number, patch: Partial<PipelineStepSpec>) => {
    onChange(steps.map((s, i) => (i === idx ? { ...s, ...patch } : s)));
  };

  const changeKind = (idx: number, kind: PipelineStepKind) => {
    const cur = steps[idx];
    const prevId = idx > 0 ? steps[idx - 1].id : undefined;
    const template = makeDefaultStep(kind, idx, prevId);
    onChange(
      steps.map((s, i) =>
        i === idx
          ? {
              ...template,
              id: cur.id || template.id,
              label: cur.label && cur.label !== KIND_META[cur.kind].label ? cur.label : template.label,
              when: cur.when || "always",
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
    onChange(copy);
  };

  const removeStep = (idx: number) => {
    onChange(steps.filter((_, i) => i !== idx));
  };

  const addStep = (kind: PipelineStepKind) => {
    const prevId = steps.length > 0 ? steps[steps.length - 1].id : undefined;
    onChange([...steps, makeDefaultStep(kind, steps.length, prevId)]);
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
    }
  };

  return (
    <div id="pipeline-builder" className="flex flex-col gap-3">
      {steps.length === 0 ? (
        <div className="rounded-sm border border-dashed border-hairline bg-surface px-3 py-4 text-center text-xs text-ink-2">
          Add typed steps below to compose a resumable pipeline with deterministic skills, AI agent steps, human checkpoints, and deliverable exports.
        </div>
      ) : (
        <ol className="flex flex-col gap-2.5">
          {steps.map((st, idx) => {
            const meta = KIND_META[st.kind] ?? KIND_META.skill;
            const KindIcon = meta.icon;
            const prevSteps = steps.slice(0, idx);
            const varChips: string[] = ["{{inputs.file}}"];
            for (const prev of prevSteps) {
              if (!prev.id) continue;
              varChips.push(`{{steps.${prev.id}.files[0]}}`);
              if (prev.kind === "checkpoint") {
                varChips.push(`{{steps.${prev.id}.human_answer}}`);
              } else {
                varChips.push(`{{steps.${prev.id}.output}}`);
              }
            }

            const argEntries = Object.entries(st.args ?? {});
            const stepErrs = validation.errorsByStep[idx] ?? [];

            return (
              <li
                key={`${idx}-${st.id}`}
                data-pipeline-step-index={idx}
                data-pipeline-step-kind={st.kind}
                className={cn(
                  "flex flex-col gap-2.5 rounded-card border bg-surface p-3",
                  stepErrs.length > 0 ? "border-dot-red" : "border-hairline"
                )}
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="flex flex-wrap items-center gap-1.5">
                    <span className="mono inline-flex size-5 items-center justify-center rounded-full bg-board text-xs font-medium text-ink">
                      {idx + 1}
                    </span>
                    <KindIcon className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                    <select
                      aria-label={`Step ${idx + 1} type`}
                      value={st.kind}
                      onChange={(e) => changeKind(idx, e.target.value as PipelineStepKind)}
                      className="focus-ring h-8 rounded-sm border border-input bg-board/60 px-2 text-xs font-medium text-ink outline-none max-md:min-h-11"
                    >
                      <option value="skill">Skill</option>
                      <option value="agent">AI Agent</option>
                      <option value="checkpoint">Human Checkpoint</option>
                      <option value="export">Export</option>
                    </select>
                    <Badge variant="outline" size="sm" className="mono text-xs text-ink-2">
                      {meta.badge}
                    </Badge>
                  </div>

                  <div className="flex items-center gap-1">
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
                  <div className="flex flex-col gap-1">
                    <label className="text-xs font-medium text-ink-2">Step ID</label>
                    <Input
                      value={st.id}
                      aria-label={`Step ${idx + 1} ID`}
                      placeholder={`step_${idx + 1}`}
                      onChange={(e) => updateStep(idx, { id: sanitizeStepId(e.target.value, `step_${idx + 1}`) })}
                      className="mono h-8 text-xs max-md:min-h-11"
                    />
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-xs font-medium text-ink-2">Step label</label>
                    <Input
                      value={st.label}
                      aria-label={`Step ${idx + 1} label`}
                      placeholder="Short plain label"
                      onChange={(e) => updateStep(idx, { label: e.target.value })}
                      className="h-8 text-xs max-md:min-h-11"
                    />
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-xs font-medium text-ink-2">Run condition</label>
                    <select
                      aria-label={`Step ${idx + 1} condition`}
                      value={st.when ?? "always"}
                      onChange={(e) => updateStep(idx, { when: e.target.value as PipelineWhen })}
                      className="focus-ring h-8 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                    >
                      <option value="always">Always run</option>
                      <option value="on_previous_ok">Only if previous succeeded</option>
                      <option value="on_anomaly">Only if anomaly flagged</option>
                    </select>
                  </div>
                </div>

                {st.kind === "skill" && (
                  <div className="flex flex-col gap-2 rounded-sm border border-hairline bg-board/40 p-2.5">
                    <div className="flex flex-col gap-1">
                      <label className="text-xs font-medium text-ink">Built-in skill</label>
                      <select
                        aria-label={`Step ${idx + 1} skill`}
                        value={st.skill_id ?? ""}
                        onChange={(e) => updateStep(idx, { skill_id: e.target.value })}
                        className="focus-ring h-8 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                      >
                        {skills.map((sk) => (
                          <option key={sk.id} value={sk.id}>
                            {sk.title} ({sk.id})
                          </option>
                        ))}
                      </select>
                    </div>

                    <div className="flex flex-col gap-1.5">
                      <span className="text-xs font-medium text-ink-2">Arguments</span>
                      {argEntries.map(([k, v]) => (
                        <div key={k} className="flex flex-wrap items-center gap-1.5">
                          <span className="mono min-w-[96px] text-xs text-ink-2">{k}</span>
                          <Input
                            value={String(v ?? "")}
                            aria-label={`Step ${idx + 1} arg ${k}`}
                            placeholder="Value or {{steps.prev.files[0]}}"
                            onChange={(e) =>
                              updateStep(idx, { args: { ...(st.args ?? {}), [k]: e.target.value } })
                            }
                            className="mono h-8 min-w-[140px] flex-1 text-xs max-md:min-h-11"
                          />
                          {workspaceFiles.length > 0 && k.includes("file") && (
                            <select
                              aria-label={`Pick workspace file for ${k}`}
                              value=""
                              onChange={(e) => {
                                if (!e.target.value) return;
                                updateStep(idx, { args: { ...(st.args ?? {}), [k]: e.target.value } });
                              }}
                              className="focus-ring h-8 rounded-sm border border-input bg-surface px-2 text-xs text-ink-2 outline-none max-md:min-h-11"
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
                      <div className="flex flex-wrap items-center gap-1.5 pt-0.5">
                        <Input
                          value={newArgKey[idx] ?? ""}
                          placeholder="Add arg name (e.g. output_title)"
                          aria-label={`New arg key for step ${idx + 1}`}
                          onChange={(e) => setNewArgKey((prev) => ({ ...prev, [idx]: e.target.value }))}
                          className="mono h-8 w-48 text-xs max-md:min-h-11"
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
                  <div className="flex flex-col gap-1.5 rounded-sm border border-hairline bg-board/40 p-2.5">
                    <label className="text-xs font-medium text-ink">Agent instruction</label>
                    <Textarea
                      rows={2}
                      value={st.prompt ?? ""}
                      aria-label={`Step ${idx + 1} agent prompt`}
                      placeholder="What should the AI agent produce using prior outputs?"
                      onChange={(e) => updateStep(idx, { prompt: e.target.value })}
                      className="text-xs"
                    />
                  </div>
                )}

                {st.kind === "checkpoint" && (
                  <div className="flex flex-col gap-2 rounded-sm border border-hairline bg-blue-soft/45 p-2.5">
                    <div className="flex flex-col gap-1">
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
                        className="text-xs max-md:min-h-11"
                      />
                    </div>
                    <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                      <div className="flex flex-col gap-1">
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
                          className="focus-ring h-8 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                        >
                          <option value="approve_reject">Approve or reject</option>
                          <option value="choice">Structured choices</option>
                          <option value="text">Free-text answer</option>
                        </select>
                      </div>
                      <div className="flex flex-col gap-1">
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
                          className="mono h-8 text-xs max-md:min-h-11"
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
                  <div className="grid grid-cols-1 gap-2 rounded-sm border border-hairline bg-board/40 p-2.5 sm:grid-cols-[minmax(0,1fr)_140px]">
                    <div className="flex flex-col gap-1">
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
                        className="mono h-8 text-xs max-md:min-h-11"
                      />
                    </div>
                    <div className="flex flex-col gap-1">
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
                        className="focus-ring h-8 rounded-sm border border-input bg-surface px-2 text-xs text-ink outline-none max-md:min-h-11"
                      >
                        <option value="docx">Word (.docx)</option>
                        <option value="xlsx">Excel (.xlsx)</option>
                      </select>
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
                      className={cn(
                        "mono rounded-chip border border-hairline bg-board/60 px-2 py-0.5 text-xs text-ink-2 transition-[color,background-color,border-color] hover:border-ink-3 hover:bg-board hover:text-ink"
                      )}
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
      </div>
    </div>
  );
}
