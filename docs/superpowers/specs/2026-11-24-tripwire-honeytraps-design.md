# Ergon Tripwire — honeytraps throughout (design)

Date: 2026-11-24 · Status: built on branch `ccr-63dd7152-ne5o7k` (owner handed over the how: "this codebase is yours")

## Intent

Ergon's promise is that each employee's agent **finishes their digital work autonomously,
with no approval ceremony**. That autonomy is exactly what attackers want to borrow: the
agent reads untrusted web pages, inbox exports, contracts and shared library skills, and it
can write files, fetch URLs, rewrite its own memory and publish code to every colleague's
agent. The goal of this work is to make hijacking that autonomy *loud and self-limiting*
without adding friction for the honest case.

Honeytraps fit the product unusually well: a decoy that no honest flow ever touches gives a
near-zero false-positive signal, so Ergon can **act** on it (block, stop the run, ban a
scanner) instead of asking the person to judge noisy alerts.

## Goals

1. Every attack surface found in the audit has at least one tripwire, and every trip is
   recorded in a tamper-evident log and shown to the affected person in plain words.
2. A tripped high-confidence trap contains the damage automatically: the request is
   blocked, the hijacked run stops, the scanner is banned.
3. Honest work is unaffected: no decoy files in the person's workspace, no new approval
   prompts on the happy path, no lock-outs of the person at their own machine.
4. Every alert says what happened, what Ergon already did, the likely source, and offers a
   one-tap follow-up (block the site / quarantine the file / allow the address / dismiss).

## Non-goals / anti-goals

- Not a general WAF or IDS. No signature feeds, no phoning home, no external webhooks.
- Anti-goal: alerts the person has to triage. Low-confidence signals only add to a run's
  threat score; they never interrupt work on their own.
- Anti-goal: decoy content in the person's files or the agent's visible outputs.
- Anti-goal: banning loopback (that is the person's own browser).

## Audit findings that shaped the design

| # | Finding | Where |
|---|---|---|
| F1 | `fetch_url` / `research` / library `host_fetch_url` follow any URL: cloud metadata (`169.254.169.254`), Ergon's own API on loopback, the LAN. Classic SSRF + exfil channel for injected instructions. | `lib.rs::fetch_url_text`, `skills.rs` host module |
| F2 | Tool results go back to the model with no trust boundary; the system prompt has no rule that content is data. | `lib.rs::run_loop`, `system_prompt` |
| F3 | Any company-library skill can `host_memory_write` the person's whole memory — a persistent prompt-injection channel into every later run. | `skills.rs::build_context` |
| F4 | `publish_skill` lets one (possibly hijacked) agent push code that reads files/memory and sends data over the network to every colleague's agent. | `execute_tool_full` |
| F5 | No security headers: the desk can be framed (clickjacking), no CSP. | `respond_bytes` |
| F6 | Refused cross-site / DNS-rebinding requests and wrong gate tokens are silently dropped: the person never learns a web page tried to drive their agent. | `refusal`, gate check |
| F7 | Sandbox escapes are refused but invisible; an agent probing `../config.json` (the model API key) looks like any typo. | `sandbox_path_resolving` callers |

## Approaches considered

1. **Classic network honeypot beside the app** (OpenCanary / T-Pot style daemon). Catches
   LAN scanners only; knows nothing about agent hijacking; another process to run. Rejected:
   off-target for a single-binary agent desk.
2. **LLM-side detection only** (Rebuff/Vigil-style canary words + injection classifiers).
   Research shows default canary-in-output checks miss most leaks (arXiv 2506.19109) and
   classifiers add a model dependency and false positives. Rejected as the primary layer;
   kept as a weak score input.
3. **Chosen: in-process deception at every trust boundary**, in the OWASP AppSensor
   tradition (detection points + automatic responses), using the strongest pattern from
   each tool researched:
   - Canarytokens → unique, attributable honeytokens per placement (here: HMAC-verifiable,
     so Ergon recognises its own tokens without a lookup and knows which placement leaked).
   - AppSensor HT1/HT2/HT3 → honey-trap *data altered*, *resource requested*, *data used*.
   - Beelzebub / HoneyMCP → a decoy that only a hijacked agent touches; lock out on trip.
   - Microsoft Spotlighting → delimit untrusted tool output with a per-run random boundary.
   - fail2ban → score-and-ban for remote scanners (never loopback).
   - SSRF-safe resolvers (agnt-tools, fetchkit) → validate *resolved* IPs inside the HTTP
     client's resolver so DNS rebinding and redirects cannot slip past.

## The design

New module `src/tripwire.rs` (one place for keys, tokens, the event log, perimeter traps,
run traps and the egress guard). Data lives in `<data>/.tripwire/` (key `0600`, issued
tokens, policy) and `<data>/security_events.json` (hash-chained).

### Honeytokens

`sk-ergon-<12 hex nonce>-<20 hex HMAC-SHA256(key, nonce)>`. Looks like an API key, is
recognisable offline (verify the MAC), and each issuance is recorded (`placement`, peer,
person, run). Any sighting — in a request header, cookie, body, or leaving through an agent
egress channel — is an `HT3` trip attributed to the placement that leaked it.

### Perimeter traps (HTTP)

- **Decoy resources (HT2)**: `/.env*`, `/.git/config`, `/config.json`, `/wp-login.php`,
  `/phpmyadmin`, `/actuator/*`, `/api/login` (Ergon has no logins), `/api/admin*`,
  `/api/internal/*`, … Each answers with plausible decoy content carrying a fresh
  honeytoken, never a real secret.
- **Breadcrumbs**: `/robots.txt` disallows `/api/internal/`; the served desk HTML carries an
  HTML comment pointing at a decoy console with a stable breadcrumb token. Only scrapers
  read those.
- **Honeytoken use (HT3)**: a served token presented back as `X-Ergon-Gate`,
  `Authorization`, cookie, `api_key` or anywhere in a body → critical, instant ban.
- **Tripwires on existing refusals**: cross-site/foreign-Host refusals (CSRF, DNS
  rebinding) and wrong gate tokens are recorded (coalesced) and scored.
- **Score-and-ban**: per remote IP; decays; escalating ban; banned peers are dropped before
  the request is read. Loopback and the unspecified address are never banned.
- **Headers**: CSP with SHA-256 hashes of the desk's own inline scripts,
  `frame-ancestors 'none'`, nosniff, `Referrer-Policy: no-referrer`, COOP/CORP, and
  `Cache-Control: no-store` on API JSON.

### Agent traps (per run)

- **Honey key bait + canary**: each run's system prompt carries a unique confidential
  "service key" the agent is told it never needs. It doubles as the run's canary.
- **Spotlighting**: tool output is wrapped in a per-run random boundary and labelled data;
  content with injection markers gets an explicit notice to the model (and an info event).
- **Egress guard** on every outbound fetch (agent tool, research workflow, library skills):
  1. honeytoken in the URL (raw, percent-decoded, base64/hex-decoded) → block, stop run;
  2. the real model key or gate token in the URL → block, stop run;
  3. SSRF-safe resolver: link-local/metadata/multicast/reserved and Ergon's own port are
     always refused; loopback/private addresses only when the host appears in the person's
     own task (provenance), is on the allow list, or `ERGON_FETCH_ALLOW_PRIVATE=1`.
     A blocked private address parks a one-tap "Allow this address" decision.
- **Sandbox tripwire**: an escape attempt aimed at credential stores or Ergon's own data
  (`../config.json`, `.ssh`, another person's folder, …) that the person did not name →
  critical, stop run. Ordinary absolute-path mistakes stay silent.
- **Honey key in any tool call** → that call is refused; egress/publish channels stop the
  run, others add score. The key is redacted from summaries.
- **High-risk phantom tools** (`bash`, `send_email`, …) → info + score only.
- **Threat score**: weak signals add up; ≥ 100 stops the run. Stopping records the run as
  unfinished and parks one `security` decision in Today with the likely source.

### Memory & library traps

- Library (non-built-in) skills cannot rewrite memory; an attempt is refused and recorded.
- An `update_memory` note carrying the honey key or injection phrasing, or introducing a
  new web address/email right after the run read injected content, is held as a
  `memory_note` decision instead of being written.
- `publish_skill` with the honey key → refused, run stopped. A skill that both reads
  files/memory and fetches URLs is held as a `publish_skill` decision; approving publishes it.

### Surfaces

- Today: security decisions with kind-specific copy and buttons (Block & quarantine /
  Dismiss; Allow address / Keep blocked; Remember it / Discard; Publish / Don't publish).
  Security events appear in "Today so far".
- Settings → Security: armed traps, recent events, log integrity, bans, allowed addresses.
- `GET /api/security`, `POST /api/security/policy` (this machine only).
- Audit bundle: a `security` section with the verified event chain and its own digest.

## Calls made by the builder (owner can revisit)

- Low-confidence signals never interrupt work; only critical trips or a score ≥ 100 stop a run.
- Private-network fetches are blocked by default unless the person's own words name the host.
- `ERGON_TRIPWIRE_NO_BAN=1` disables banning; `ERGON_TRUST_PROXY=1` uses `X-Forwarded-For`.
- No outbound alerting (no webhook): privacy first, matching "never phones home" spirit.
- Version bump to 0.8.0 with a CHANGELOG entry.
