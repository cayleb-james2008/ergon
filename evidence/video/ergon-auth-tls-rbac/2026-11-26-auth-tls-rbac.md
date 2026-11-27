# Video Proof Sidecar — `ergon-auth-tls-rbac`

- **Surface / App:** `ergon-auth-tls-rbac` (Gap 1 / P0: Multi-user local accounts with PBKDF2-HMAC-SHA256, persisted opaque HttpOnly/Secure sessions, optional OIDC PKCE, HTTPS/TLS via `rustls` + auto self-signed certs, centralized RBAC, strict cross-user isolation, and `/phone` behind TLS + login)
- **Baseline Tag / Branch:** `v0.38.0` (`feat/mvp-auth-tls-rbac`, base `28c3a7354`)
- **Host OS:** `Linux outis-desktop 7.2.15-4-omarchy #1 SMP PREEMPT_DYNAMIC Sun, 22 Nov 2026 15:33:45 +0000 x86_64 GNU/Linux` (Omarchy / Arch Linux)
- **Result:** `PASS`
- **Worktree Video Path:** `evidence/video/ergon-auth-tls-rbac/2026-11-26-auth-tls-rbac.mp4` (gitignored per large-file policy)
- **Linked Deliverables Path:** `~/Work/deliverables/ergon-video-evidence-2026-11-26/ergon-auth-tls-rbac/2026-11-26-auth-tls-rbac.mp4`
- **Video Metadata:** `32.05s` (`00:32`), `682,974` bytes (`667 KiB`), `1440x900`, `h264`, `yuv420p`, `20 fps`, uncut single Xvfb take (`DISPLAY=:93`, `GDK_SCALE=1`, `GDK_DPI_SCALE=1`, `XDG_SESSION_TYPE=x11`)
- **Video SHA-256:** `b4c5a8dd486da8df09d43341f945e30927975a95c7662e7392a6df1cd53c49cb`

## Exact Commands & Inputs

- **Cold TLS Launch Command:**
  `ERGON_DATA_DIR=./demo-data ERGON_PORT=18931 ERGON_NO_AUTO_MODEL=1 ERGON_NO_OPEN=1 ergon --tls --no-open &`
- **Unauthenticated & Plaintext Rejection Probes:**
  - `curl -s -i http://127.0.0.1:18931/api/status | head -n 5` (`HTTP/1.1 400 Bad Request` — plaintext HTTP rejected on TLS port)
  - `curl -sk -i https://127.0.0.1:18931/api/status | head -n 6` (`HTTP/1.1 401 Unauthorized` — authentication required)
- **Non-Admin RBAC & Cross-User Isolation Probes (as `maya`, role `installer`):**
  - `POST https://127.0.0.1:18931/api/auth/login` -> `200 OK` with `Set-Cookie: ergon_session=<redacted>; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=43200`
  - `POST https://127.0.0.1:18931/api/people/switch` (`{"id":"alex-rivera"}`) -> `HTTP/1.1 403 Forbidden`
  - `GET https://127.0.0.1:18931/api/auth/users` -> `HTTP/1.1 403 Forbidden`
  - `GET https://127.0.0.1:18931/api/invoices` -> `HTTP/1.1 403 Forbidden`
  - `POST https://127.0.0.1:18931/api/clients` -> `HTTP/1.1 403 Forbidden`
  - `ls -la ./demo-data/auth_users.json ./demo-data/auth_sessions.json` -> mode `0600` (`-rw-------`), zero plaintext password matches (`pbkdf2_sha256$120000`)
- **Inputs Used:** Clean empty `./demo-data` with synthetic demo accounts (`alexadmin` / `"Alex Rivera"` / `admin`, and `maya` / `"Maya Lin"` / `installer`), zero real credentials, zero mocked API responses, zero browser console errors (`console_errors: []`).

## Steps Shown & Timestamps

| Timestamp | Event |
|---|---|
| `00:00` | Continuous uncut capture starts on `DISPLAY=:93` (`1440x900`, 1:1 DPI); cold-launches `ergon 0.38.0 --tls --no-open` on port `18931` against empty `./demo-data`, auto-generating self-signed TLS cert (`cert.pem` `0644`, `key.pem` `0600`) |
| `00:02` | Confirms plaintext HTTP (`http://127.0.0.1:18931/api/status`) is rejected (`HTTP/1.1 400 Bad Request`) and unauthenticated HTTPS (`/api/status`) is gated (`HTTP/1.1 401 Unauthorized`) |
| `00:05` | Opens real headed Chromium (`1440x900`, `--ozone-platform=x11 --force-device-scale-factor=1`) to `https://127.0.0.1:18931/`, showing the **Set up Ergon Admin** screen with **HTTPS / TLS** pill |
| `00:08` | Completes first-run admin setup (`alexadmin`, `Alex Rivera`, `alex@fieldops.example`), landing inside Alex Rivera's authenticated HTTPS workspace |
| `00:10` | Opens **Settings** -> **Account & Access Control** -> creates second non-admin account `maya` (`Maya Lin`, role `installer`), confirming toast and `Team Accounts & Roles (2)` with isolated workspace `maya-lin` |
| `00:15` | Signs out from Admin -> immediately returns to **Sign in to Ergon** (`HTTPS / TLS`) gate |
| `00:17` | Closes Chromium to reveal terminal in the same uncut take; logs in as `maya` (`installer`) over HTTPS/TLS confirming `HttpOnly; SameSite=Lax; Secure` cookie |
| `00:19` | Executes live non-admin RBAC & cross-user isolation probes proving `HTTP/1.1 403 Forbidden` on `POST /api/people/switch` (targeting `alex-rivera`), `GET /api/auth/users`, `GET /api/invoices`, and `POST /api/clients`, plus verifies `0600` file modes on `auth_users.json` & `auth_sessions.json` with `0` plaintext passwords |
| `00:22` | Reopens fresh Chromium context directly to `https://127.0.0.1:18931/phone`, proving `/phone` sits behind the exact same HTTPS/TLS + login gate (**HTTPS / TLS** and **Field / Phone View** pills) |
| `00:25` | Signs into `/phone` as `maya` (`installer`) over HTTPS/TLS, loading Maya Lin's isolated field workspace |
| `00:27` | Opens **Settings** on `/phone` confirming `HTTPS / TLS` + `installer` role badge, Workspace switcher restricted strictly to `Maya Lin (Active)` (no other users exposed, no admin controls) |
| `00:30` | Closes Chromium, stops `ergon` cleanly via tracked `ergon.pid`, and prints final `PASS` confirmation |
| `00:31` | Continuous uncut capture ends cleanly |

## Observed Notes

- Zero browser console or page errors (`console_errors: []`).
- Rust automated coverage (`cargo test --locked -- --test-threads=1`: `302/302` PASS across `18` test binaries, including `tests/auth_tls_rbac.rs` `6/6`, `auth::tests` `3/3`, `tls::tests` `1/1`, and `e2e` TLS default installer boot) also verifies OIDC Discovery + Auth Code + PKCE (`S256`) + state/nonce validation + userinfo mapping end-to-end against a local loopback OIDC test server, admin break-glass `ERGON_GATE_TOKEN` auditing & non-elevation, and CLI `ergon user add/passwd/list` + `ergon tls init`.
