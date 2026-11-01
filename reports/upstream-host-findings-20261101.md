# UPSTREAM HOST FINDINGS — DSH desktop host (for the Nox upstream pack, 1 Nov 2026)

Two host properties observed today on `outis-desktop`, app `@deepseek-ai/dsh-desktop 0.2.2-alpha.2`
(electron 44.7.0, node 24.21.0, linux x64). They are the same complaint: the host cannot survive, or even
explain, a component's death. Evidence is local (crash logs, timings, journal); no exploit or privilege angle.

## H1 — In-process native image decode can kill the host silently
- Four host stops today, each 11–50 s after a PNG screenshot written by lane work:
  10:04:14.68 `studio-focused-100414.png` → stop 10:05:04.78 (+50 s)
  11:02:15.23 `studio-ws3-110214.png` → stop 11:02:37.67 (+22 s)
  11:22:28.52 `pre-open-112227.png` (2.5 MB) → stop 11:22:47.73 (+19 s)
  11:36:33.85 `verify-post-crash-113632.png` → stop 11:36:44.68 (+11 s)
  (Two further screenshots, 10:01:41 and 11:00:59, did not stop the host — not every read kills, but every stop has a read just before it.)
- Every crash log tail carries `(node:…) [SharpElectronLinux] Warning: Binaries provided by Electron for use on Linux may be incompatible with sharp`. sharp is the image-processing library.
- Crash profile: the shell's ChildProcess close handler fires (main.js:9837 `maybeClose`); the host child is gone; no JS exception, no V8 heap OOM, no kernel OOM/segfault in the journal, no Electron crashpad entry, no renderer error. That is the signature of a native crash inside image decode.
- Fix direction: decode images out-of-process (or a guarded decoder process) so a bad/large image cannot take the host down; surface decode failures as tool errors, not host death.

## H2 — In-process plugin execution with stderr-tail-only crash evidence
- Plugins run in-process with no sandbox (per the component audit): any plugin can take the whole host down.
- The crash log records only the host's stderr TAIL, not its exit code/signal/full stderr. This produced a
  full misdiagnosis today: the tail was the memory plugin's STARTUP output (it prints first and nothing else
  printed afterwards), so the memory plugin looked guilty; the actual trigger was H1. The lesson: a log tail
  is not a cause.
- Fix direction: (a) crash evidence should include exit code/signal, full stderr (bounded), and the last
  N in-flight tool/plugin operations with timestamps; (b) plugin work should run in a child process or be
  restartable so a plugin crash degrades one component, not the host.

## Suggested upstream report shape
Two issues: "host dies silently on image decode (sharp/Electron-Linux)" and "host crash evidence is a stderr tail
only + in-process plugin execution makes every component a host-crash risk". Attach the four crash logs
(electron-user-data/logs/crash-2026-11-01T15-05-04, 16-02-37, 16-22-47, 16-36-44), the screenshot timings, and the
SharpElectronLinux line. Confirmation test (per Nox 164500Z ruling): throwaway profile/session only, nothing in
flight, one large PNG then a tiny PNG control; file the result either way. Not run on the live host while lanes
are in flight.
