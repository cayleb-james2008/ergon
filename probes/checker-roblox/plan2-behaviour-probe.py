"""Independent checker probe for Plan 2 (29a626b): exercises the real runner gate against the repo's own test doubles.
Fresh copy at /tmp/checker-roblox-plan2/repo (git HEAD 29a626b, tracked tree clean). No maker working tree is touched."""
import contextlib, io, json, sys, unittest.mock as mock
REPO = "/tmp/checker-roblox-plan2/repo"
sys.path.insert(0, REPO + "/tools")
sys.path.insert(0, REPO + "/tests/tools")
import vinegar_run as V
import test_vinegar_run as T

out = {"constants": {"WORLD_READY_TIMEOUT": V.WORLD_READY_TIMEOUT, "WORLD_READY_POLL": V.WORLD_READY_POLL,
                     "Luau_has": {k: (k in V.WORLD_READY) for k in ("WorldReady", "WorldSeed", "WorldLayoutVersion", "Sky", "Atmosphere")},
                     "Luau_gates_on_technology": ("Technology" in V.WORLD_READY and "lightingTechnology=if ok" in V.WORLD_READY),
                     "Luau_requires_technology": False}}

# --- 1. pass path through the real run_fixture state machine (repo's fake MCP)
t = T.FixtureTests("test_world_ready_gate_reads_before_the_first_capture_and_records_its_trace")
t.setUp()
try:
    fake, ctx = t.fare(T.FareScenario(polls=8))
    code, result = t.run_fixture(ctx, "moving")
    out["pass_path"] = {"exit": code, "worldReady": result.get("worldReady"),
                        "captures": fake.calls.count("screen_capture"),
                        "ready_reads": fake.scenario.ready_calls,
                        "captures_before_each_ready_read": fake.scenario.ready_capture_counts,
                        "result_keys_with_worldReady": "worldReady" in result}
finally:
    t.doCleanups()

# --- 2. expiry path: bounded 0.05 s, never-ready
t2 = T.FixtureTests("test_world_ready_gate_fails_closed_with_the_named_message_and_no_capture")
t2.setUp()
try:
    with mock.patch.object(V, "WORLD_READY_TIMEOUT", 0.05):
        fake2, ctx2 = t2.fare(T.FareScenario(not_ready=lambda n: {
            "rootFound": True, "ready": False, "seed": None, "layoutVersion": None,
            "lightingTechnology": None, "lightingReadable": False,
            "skyPresent": False, "atmospherePresent": False}))
        code2, result2 = t2.run_fixture(ctx2, "moving")
    frames = sorted(str(p) for p in (ctx2.evidence).glob("*/frames/*"))
    out["expiry_path"] = {"exit": code2, "error": result2.get("error"),
                          "worldReady": result2.get("worldReady"),
                          "captures": fake2.calls.count("screen_capture"),
                          "frame_sidecar_files": frames,
                          "abort": result2.get("abort"),
                          "report_json_written": bool(list((ctx2.evidence).glob("*/report.json")))}
finally:
    t2.doCleanups()

# --- 3. poll cadence + bound with a fake clock (proves 0.25 s re-poll and the timeout expiry, incl. default 30 s)
class Clock:
    t = 1000.0
    def monotonic(self): return self.t
clock = Clock()

class FakeCtx:
    identity = None
    session = None
    def remaining(self, timeout): return FakeCtx.budget
    def sleep(self, s):
        FakeCtx.sleeps.append(s)
        clock.t += s

def state_factory(ready_after=None):
    calls = {"n": 0}
    def read(_ctx):
        calls["n"] += 1
        clock.t += 0.0
        if ready_after is not None and calls["n"] > ready_after:
            return {"rootFound": True, "ready": True, "seed": 2134783819, "layoutVersion": 2,
                    "lightingTechnology": None, "lightingReadable": False, "skyPresent": True, "atmospherePresent": True}
        return {"rootFound": True, "ready": False, "seed": None, "layoutVersion": None,
                "lightingTechnology": None, "lightingReadable": False, "skyPresent": False, "atmospherePresent": False}
    return read

def run_gate(ready_after, timeout, budget):
    FakeCtx.sleeps = []
    FakeCtx.budget = budget
    clock.t = 1000.0
    c = FakeCtx()
    with mock.patch.object(V, "world_ready_state", state_factory(ready_after)), \
         mock.patch.object(V.time, "monotonic", clock.monotonic):
        try:
            return {"trace": V.await_world_ready(c, timeout), "sleeps": list(FakeCtx.sleeps), "elapsed": round(clock.t - 1000.0, 3)}
        except V.WorldNotReady as exc:
            return {"raised": str(exc), "trace": exc.trace, "sleeps": list(FakeCtx.sleeps), "elapsed": round(clock.t - 1000.0, 3)}

out["fake_clock"] = {
    "ready_after_3_polls": run_gate(3, 1.0, 30),
    "never_ready_timeout_1s": run_gate(None, 1.0, 30),
    "never_ready_default_30s": run_gate(None, None, 120),
    "never_ready_budget_5s": run_gate(None, None, 5),
}
print(json.dumps(out, indent=2, default=str))
