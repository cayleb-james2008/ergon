# LANE 4 — browser control: the installed background-browser plugin, named + verified + absorb/remove assessment

Filed 2026-11-01T15:55Z by `fleet-lead` (Team B, Lane 4) under Nox's browser-control acceptance amendment
(150600Z): control of Cayleb's own Chrome **and** the integrated background browser behind the chat, mirrored
into the right column, are both acceptance requirements of the finalised single plugin.

## 1. Identification — BY NAME, from Cayleb's own desktop profile inventory

**`dsh-inner-browser-use` version `0.41.0`** — the plugin this session's toolset uses (the `browser_open` /
`browser_view` / `browser_shot` / `browser_login` / `browser_media` family).

- Found in the **desktop profile inventory**:
  `…/apps/desktop/.desktop-build/development/home/profiles/desktop/package.json` — dependencies include
  `"dsh-inner-browser-use": "0.41.0"`, and the profile `dsh.profile.bundles` list mounts `dsh-inner-browser-use`.
- Its data dir on this machine: `~/.dsh/inner-browser-use/` (Chrome profile + `asks.log` + encrypted logins),
  matching the tool descriptions in this session's own toolset.
- Description (verbatim): *"An agent-owned Chromium sidecar mirrored into the DeepSeek Harness right column:
  the agent drives it, the person watches and can take the mouse, and logins survive restarts."*
- Its `dsh.client` manifest injects into `@deepseek-ai/dsh-client-ui-sidebar-right` → the right-column mirror
  panel is **its** mechanism; it is not a different browser surface.

## 2. Licence and source verification (every adopted component gets this)

| Field | Value |
|---|---|
| Licence | **MIT** (`package.json` + `LICENSE`: "MIT License, Copyright (c) 2026 regrevia"); MIT permits absorption with attribution |
| Registry | npm `dsh-inner-browser-use@0.41.0`, maintainer `regrevia` |
| `dist.shasum` (registry) | `5659e0977568188c6add8e82982db4ab4cb2010f` — **matches the downloaded tarball** (`sha1sum 5659e097…`) |
| `dist.integrity` (registry) | `sha512-zZZyHwj/MD4Hj0D39sfA6geN64I4sIv8yHszezb43El1DIIsybYaOASjwjRZeU/CbBOoAKow3QwO8PzHoHPVwg==` — **recomputed from the tarball, matches** |
| Published tarball | 225,318 B, 32 files (`/tmp/ibr-verify/ibr.tgz`, retained digest in this file) |
| Installed vs published | **byte-identical** — `diff -rq published/ installed/` → no differences (`rc=0`) |
| Installed file digests | `index.js 8aa617d9…`, `client.js 548ce966…`, `read.js c365a1fc…`, `media.js 01da24d7…`, `package.json 6fa37c3a…`, `cordis.patch.yml 84c63c50…` (installed = published) |
| Installed size / count | 784 KB on disk; client UI + tools + read/media/scripts/runtime/docs |
| Terms | MIT — no subscription/terms conflict; individual third-party maintainer; supply-chain control = pin exact version + integrity, no auto-update |

**Verdict: identified, verified, licence-compatible.** This is the component the amendment names; no substitute
was used.

## 3. What it already satisfies vs what is missing (acceptance mapping)

| Requirement (Nox 150600Z) | Status |
|---|---|
| (b) integrated background browser behind the chat, mirrored into the right column, agents do their browser work there | **The plugin is exactly this** and is live in Cayleb's desktop profile today. What is missing for the *finished single plugin* is that it must ship **inside** `@cayleb/dsh-fleet` (absorb-or-remove) rather than as a separate personal-profile dependency. |
| (a) control of Cayleb's own Chrome | **NOT provided by this plugin — GAP NAMED.** It runs its **own** Chromium profile (separate from Cayleb's Chrome/its session and tabs). Driving *his* Chrome requires a CDP-attach route (`--remote-debugging-port` on a profile he starts) or a bridge into his running Chrome — a different capability with its own consent/security questions (his live profile, cookies, saved logins). It must not be silently substituted or faked. |
| Right-column mirror for the fleet | The plugin's own client inject is the mechanism; absorbing it brings the mirror with it. |

## 4. Absorb-or-remove assessment

**Option A — absorb (recommended):** vendor the byte-pinned published artefact into the fleet package subtree
(e.g. `package/vendor/dsh-inner-browser-use@0.41.0/`), carry its MIT `LICENSE` + copyright notice as `NOTICE`
in the package, mount its bundle patch + client inject from the fleet package's own profile patch, and add a
release test that the vendored bytes hash-equal the pinned artefact. Effect: one install carries the browser
surface; the personal profile can keep or drop its own copy independently. Cost: package grows ~225 KB
compressed; one mount row; one client inject; licence file; tests.

**Option B — declared dependency (minimal change):** the single fleet package declares
`dsh-inner-browser-use@0.41.0` as a dependency (exact pin) and mounts it in the fleet profile. One install still
carries it, but it lives in `node_modules` rather than the package tree. Acceptable only if "single plugin"
means one installed bundle rather than one source tree — that is Nox's/Lead's call.

**Option C — remove:** if adoption is refused, the background-browser acceptance property fails and must be
reported as a named gap; the plugin stays in Cayleb's personal profile untouched.

**Fences for all options:** no write to Cayleb's personal desktop profile except through Lane 1's ruled path
(Rule W); the fleet package gets its own mounted copy; exact version + integrity pinned; MIT notice carried;
no auto-update; no prototype code imported.

## 5. Named gaps / decisions to surface

1. **Cayleb's own Chrome is not covered** — named gap; needs a CDP-attach capability decision (his profile,
   consent, security). Not built, not substituted.
2. **Absorb vs dependency** — recommendation A (absorb with MIT notice + pinned bytes); B acceptable if the
   single-plugin property is read as one installed bundle. Owner/Nox ruling if the rule is meant strictly.
3. **Supply chain** — individual maintainer; pinned + integrity-verified tarball, no auto-update; vendor copy
   reviewed at the pinned version only.
4. **Personal-profile coexistence** — the desktop profile currently mounts `dsh-inner-browser-use@0.41.0`; the
   fleet package's absorbed copy is a separate mount for the fleet profile. No change to his personal profile
   is made by this lane.

## 7. RECONCILIATION (added 2026-11-01T17:05Z)

The audit candidate `dsh-builtin-browser` (wqty123/dsh-browser, 0.5.2) and this installed component are **different
components for the same gap**. Head-to-head comparison filed:
`board/reports/team-b-browser-control-reconciliation.md` — including the decisive leg-(b) finding (**neither**
component controls Cayleb's *running* Chrome; both launch his installed Chrome/Edge binary with a plugin-owned
profile) and the candidate's **main.js desktop-bridge patch** (the audit's crash class; Windows-verified only;
dev-build host here). **Neither is absorbed; Nox rules the pick.** The named gaps in §3/§5 above stand.

## 6. Evidence retained

`/tmp/ibr-verify/ibr.tgz` (225,318 B; sha1 `5659e097…`; sha512-b64 `zZZyHwj/…`), `/tmp/ibr-verify/x/package/`
(extracted published tree), installed profile path above, `~/.dsh/inner-browser-use/` (data dir),
`node_modules/dsh-inner-browser-use/` (installed copy). Digests quoted in §2.
