#!/usr/bin/env node
// Verifies browser metering formula & $0/hr unassigned hourly fallback parity
// against Rust's `src/records.rs:2143-2179` and `src/lib.rs:3282-3291`.
//
// Modes:
//   node frontend/scripts/test-metering-parity.mjs               -> runs built-in assertions
//   node frontend/scripts/test-metering-parity.mjs --eval-vectors -> reads JSON vectors from stdin, prints JSON results

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  FORMULA_ESTIMATE_TOOLTIP,
  computeBillableValueUsd,
  computeMinutesSaved,
  resolveEffectiveHourlyRate,
  round2,
  round4,
} from "../src/lib/meteringFormula.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "../..");

if (process.argv.includes("--eval-vectors")) {
  const raw = fs.readFileSync(0, "utf8");
  const vectors = JSON.parse(raw);
  const results = vectors.map((v) => {
    const minutes_saved = computeMinutesSaved(
      Boolean(v.finished),
      Boolean(v.answered),
      Number(v.ok_steps),
      Number(v.files_written),
      Number(v.completion_tokens)
    );
    const rate =
      v.engagement !== undefined || v.client !== undefined
        ? resolveEffectiveHourlyRate(v.engagement ?? null, v.client ?? null)
        : Number(v.hourly_rate_usd ?? 0);
    const billable_value_usd = computeBillableValueUsd(minutes_saved, rate);
    return {
      minutes_saved,
      effective_hourly_rate_usd: rate,
      billable_value_usd,
    };
  });
  process.stdout.write(JSON.stringify(results));
  process.exit(0);
}

// 1. Hourly fallback parity ($0/hr when unassigned, matching src/lib.rs & src/pipelines.rs)
assert.equal(resolveEffectiveHourlyRate(null, null), 0, "unassigned must fall back to $0/hr");
assert.equal(resolveEffectiveHourlyRate({}, {}), 0, "empty engagement/client must fall back to $0/hr");
assert.equal(
  resolveEffectiveHourlyRate({ hourly_rate_usd: 0 }, { default_rate_usd: 0 }),
  0,
  "zero rates must return $0/hr"
);
assert.equal(
  resolveEffectiveHourlyRate({ hourly_rate_usd: -50 }, { default_rate_usd: Number.NaN }),
  0,
  "negative/NaN rates must fall back to $0/hr"
);
assert.equal(
  resolveEffectiveHourlyRate({ hourly_rate_usd: 0 }, { default_rate_usd: 185 }),
  185,
  "zero engagement rate inherits client default rate"
);
assert.equal(
  resolveEffectiveHourlyRate({ hourly_rate_usd: 220 }, { default_rate_usd: 185 }),
  220,
  "positive engagement rate overrides client default rate"
);

// 2. computeMinutesSaved exact branch parity with src/records.rs:2143-2164
assert.equal(computeMinutesSaved(false, false, 0, 0, 1000), 0);
assert.equal(computeMinutesSaved(false, false, 2, 1, 5000), 8); // 2*2 + 1*4 = 8 (tokens ignored)
assert.equal(computeMinutesSaved(true, true, 0, 0, 0), 2);
assert.equal(computeMinutesSaved(true, true, 0, 0, 500), 4); // 2 + 500/250
assert.equal(computeMinutesSaved(true, true, 0, 0, 10_000), 12); // capped at +10
assert.equal(computeMinutesSaved(true, false, 1, 0, 0), 7); // pipeline 0 files
assert.equal(computeMinutesSaved(true, false, 1, 1, 0), 15); // pipeline 1 file
assert.equal(computeMinutesSaved(true, false, 1, 2, 0), 23); // pipeline 2 files
assert.equal(computeMinutesSaved(true, false, 3, 1, 480), 24.92); // 3 + 12 + 8 + 1.92
assert.equal(computeMinutesSaved(true, false, 2, 2, 5000), 42); // 3 + 8 + 16 + 15 cap

// 3. computeBillableValueUsd exact parity with src/records.rs:2166-2179
assert.equal(computeBillableValueUsd(23, 0), 0);
assert.equal(computeBillableValueUsd(23, -100), 0);
assert.equal(computeBillableValueUsd(0, 220), 0);
assert.equal(computeBillableValueUsd(Number.NaN, 220), 0);
assert.equal(computeBillableValueUsd(24.92, 220), 91.3733);
assert.equal(computeBillableValueUsd(15, 180), 45);
assert.equal(round2(1.235), 1.24);
assert.equal(round4(1.23456), 1.2346);

// 4. Static regression checks on webEngine.ts (no $150 fallback or hardcoded regex bucket table)
const webEngineSrc = fs.readFileSync(
  path.join(repoRoot, "frontend/src/lib/webEngine.ts"),
  "utf8"
);
assert.ok(
  !webEngineSrc.includes("function estimateMinutesSaved("),
  "webEngine.ts must not retain legacy regex estimateMinutesSaved()"
);
assert.ok(
  !/function effectiveHourlyRate[\s\S]{0,400}return 150;/.test(webEngineSrc),
  "webEngine.ts effectiveHourlyRate must not fall back to $150/hr"
);
assert.ok(
  webEngineSrc.includes("resolveEffectiveHourlyRate") &&
    webEngineSrc.includes("computeMinutesSaved") &&
    webEngineSrc.includes("computeBillableValueUsd"),
  "webEngine.ts must delegate to shared meteringFormula helpers"
);
assert.equal(
  FORMULA_ESTIMATE_TOOLTIP,
  "Formula estimate — not yet calibrated to stopwatch timings"
);

console.log("PASS: meteringFormula.ts & webEngine.ts match src/records.rs ($0/hr fallback + exact minutes/billable parity)");
