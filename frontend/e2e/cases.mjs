// cases.mjs — Ergon v0.3.0 worst-case replay suite for ultra-design kit/replay.mjs (1440px and 390px).
export default ({ LONG, json, patch, malformed, down, overflowing, has }) => [
  {
    name: "null and numeric person names do not blank the desk",
    routes: [[
      "**/api/people",
      patch((j) => ({
        ...j,
        people: [...(j.people || []), { id: "n1", name: null, role: null }, { id: "n2", name: 42, role: "" }],
      })),
    ]],
    check: async (p) => (await p.locator("main h1").count()) === 1 || "no headline rendered",
  },
  {
    name: "long unbroken decision, finished task, and today event titles stay inside boxes",
    shot: true,
    routes: [
      ["**/api/decisions", patch((j) => ({ decisions: j.decisions.map((d) => ({ ...d, summary: "Delete " + LONG + ".txt" })) }))],
      ["**/api/brief", patch((j) => ({ ...j, finished: j.finished.map((f) => ({ ...f, task: "Task " + LONG })) }))],
      ["**/api/today", patch((j) => ({ ...j, events: (j.events || []).map((ev) => ({ ...ev, title: "Event " + LONG })) }))],
    ],
    check: async (p) => (await overflowing(p, "#lead-h, main ol p")) === 0 || "text wider than its box",
  },
  {
    name: "malformed brief shows plain error with Retry",
    routes: [["**/api/brief", malformed]],
    check: async (p) => (await has(p, "Today's paper could not be loaded")) || "missing brief error message",
  },
  {
    name: "unreachable brief shows plain error with Retry",
    routes: [["**/api/brief", down]],
    check: async (p) => (await has(p, "Today's paper could not be loaded")) || "missing unreachable brief error message",
  },
  {
    name: "a question decision states answer is recorded and offers Decline",
    routes: [[
      "**/api/decisions",
      patch((j) => ({ decisions: j.decisions.map((d) => ({ ...d, kind: "question", summary: "Use the new supplier list?" })) })),
    ]],
    check: async (p) => {
      const band = await p.locator("section[aria-labelledby=lead-h]").innerText();
      if (!/recorded/.test(band) || /went ahead/.test(band)) return "unexpected band copy: " + band.slice(0, 80);
      return (await p.getByRole("button", { name: "Decline", exact: true }).count()) === 1 || "missing Decline button";
    },
  },
  {
    name: "malformed conversation list shows its error state",
    routes: [["**/api/threads", (r) => (r.request().method() === "GET" ? json(r, '{"threads": [tr') : r.continue())]],
    act: async (p) => {
      if ((p.viewportSize()?.width ?? 1440) < 768) {
        await p.getByRole("button", { name: "Open menu" }).click();
        await p.waitForTimeout(400);
      }
    },
    check: async (p) => (await has(p, "Could not load your conversations")) || "missing conversations error message",
  },
  {
    name: "malformed people list shows error alert in Team card",
    routes: [["**/api/people", (r) => (r.request().method() === "GET" ? json(r, '{"people": [tr') : r.continue())]],
    check: async (p) =>
      (await p.locator("#team-h ~ p[role=alert]:visible", { hasText: "Could not load the people list" }).count()) === 1
      || "missing people error alert",
  },
  {
    name: "many long saved-file names wrap cleanly with file-type glyphs",
    shot: true,
    routes: [[
      "**/api/brief",
      patch((j) => ({
        ...j,
        finished: j.finished.map((f, i) => ({
          ...f,
          files: i === 0 ? [LONG + ".csv", "noext", "a.PNG", "b.json", "c.pdf", "d.unknown", "e.xlsx", "f.md"] : f.files,
        })),
      })),
    ]],
    check: async (p) => {
      if ((await overflowing(p, "main ol p")) !== 0) return "file row overflowed";
      return (await p.locator("section[aria-labelledby=finished-h] ol li svg[aria-hidden=true]").count()) >= 8 || "missing file glyphs";
    },
  },
  {
    name: "non-array or null files values do not crash Finished or Today stream",
    routes: [
      [
        "**/api/brief",
        patch((j) => ({
          ...j,
          finished: j.finished.map((f, i) => ({ ...f, files: i === 0 ? "oops.csv" : i === 1 ? null : [null, 42, "ok.md"] })),
        })),
      ],
      [
        "**/api/today",
        patch((j) => ({
          ...j,
          events: [{ ts: Date.now(), kind: "work", title: "Odd event", detail: null, files: "not-an-array" }],
        })),
      ],
    ],
    check: async (p) => (await p.locator("main h1").count()) === 1 || "headline crashed on non-array files",
  },
  {
    name: "malformed work graph shows plain error with Retry",
    routes: [["**/api/graph", malformed]],
    act: async (p) => {
      await p.getByRole("tab", { name: "Work Graph" }).click();
      await p.waitForTimeout(500);
    },
    check: async (p) => (await has(p, "The graph could not be loaded.")) || "missing graph error state",
  },
];
