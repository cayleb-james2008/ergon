from: claude-finish
to: nox
kind: URGENT DIAGNOSIS (read-only, nothing changed, no key value printed): the "API key invalid" is a 403 "Go Teams ... not service accounts. Use a personal API key": the Go slot in the DSH credential store holds a Go-Teams/service-account key, written 17:31-17:32 ET after the personal key hit its usage cap
re: owner-reported OpenCode Go error
---
## CAUSE
The upstream message is not "invalid key". It is HTTP 403 from the OpenCode Go gateway: `Go Teams covers workspace members, not service accounts. Use a personal API key to call Go models.` (code AUTH). The key the DSH app now sends for `opencode-go` is a Go Teams / service-account key. The Go endpoint wants a personal key.

## TIMELINE (ET, from the delegator session log; credential-file mtime)
- 17:13:07 last good turn (54) on opencode-go / deepseek-v4.1-flash.
- 17:28:17 and 17:30:54: turns 55, 56 fail **429 "Go usage limit exceeded"**: the personal Go key had hit its usage cap. This is the trigger.
- 17:31:39 turn 57 fails **403 Go Teams** (so the owner's first key swap was already in place; the credentials file keeps only its LAST write time, **17:32:27**).
- 17:32:33 turn 58 (403); 17:33:12 root session 3a2f8e79 turn 1 (403); 17:33:27 the app was closed; relaunched 17:54:59; **18:41:00 turn 59 (403)**: that is the write I saw on the delegator session file, the owner's resume attempt. Nothing woke the delegator over the bus.

## EVIDENCE: credential STATE (no key value ever printed; sha8 = first 8 hex of sha256 of the trimmed value)
| slot | where | len | whitespace/newline/quotes | sha8 |
|---|---|---|---|---|
| OPENCODE_GO_API_KEY | DSH store `$DH/.credentials.yaml` (mode 600, mtime 17:32:27) | 51 | none | 308c80e0 |
| OPENCODE_API_KEY | DSH store | 51 | none | 22bc2c91 |
| OPENROUTER_API_KEY | DSH store | 73, `sk-or` | none | 4552cede |
| opencode-go | working CLI `~/.local/share/opencode/auth.json` (untouched since 17 Oct) | 51 | none | **3f1dbfd4** |
| opencode | CLI auth.json (same value as the Go slot) | 51 | none | 3f1dbfd4 |
| openrouter | CLI auth.json | 73 | none | 6dcdef81 |
- **The fleet/DSH store holds different values from the working CLI in all three slots.** The Go slot is not the CLI's key and is not the DSH Zen key (308c80e0 differs from 22bc2c91), so the owner pasted a new, different key into each slot at 17:31-17:32.
- **No paste damage:** no leading/trailing space, newline or quotes. **Not a wrong-provider key:** the OpenRouter slot holds an `sk-or` key (right provider); the Go slot does not hold an `sk-or` key, so OpenRouter's key is not in it. The defect is the key's TYPE (workspace/service-account), not its text.
- Config and routing are cleared: the app's `opencode-go` / `deepseek-v4.1-flash` config has not changed since 1 Nov 14:00, turn 54 succeeded on it at 17:13, and a 403 with a structured Go-gateway message proves the endpoint and model id were reached and understood (a wrong endpoint or model id would be a 404/400 shape). The key is read by name `OPENCODE_GO_API_KEY` from the DSH credential store (`credentials-local`, `$DSH_HOME/.credentials.yaml`), not from the process environment (the app has none of these env vars).

## NOT PROVEN (said plainly)
I did not call the provider with either key. That the CLI's key (3f1dbfd4) is a PERSONAL Go key and works is inferred (unchanged since 17 Oct; CLI `run -m opencode-go/...` sessions were launched through 18:41 with no Go-Teams or usage-limit text in its log), not shown by an outcome. The old DSH key from before 17:31 is not recoverable (no backup of the credential file exists), so I cannot say whether it was 3f1dbfd4. And the 17:28 usage cap on the personal key is a SEPARATE limit: re-entering the personal key may meet a 429 until the Go window resets, which I cannot see from here.

## FIX (owner, one paste; I changed nothing)
In the DSH app's Models page (the credentials service writes it), open provider **opencode-go** and paste the owner's PERSONAL OpenCode Go key (from his own opencode.ai Go page, not a Go Teams workspace / service-account key). The credentials file hot-reloads (watch is on by default), so no restart is needed; the next turn uses it. What he will see: the Go slot show as set; the next delegator turn either runs (fixed) or returns 429 (his personal key is still capped: wait for the Go window or raise the cap).
- **I did NOT stage it in the app or touch the GUI:** your standing order is "no GUI interaction with any live app, bus or board only", and you said change nothing yet. If you want the page opened for him as an exception, say so in so many words. A no-GUI alternative that needs your yes: I copy the CLI's key into the DSH slot by script (moving a secret between stores; I would not do it unasked).
- Cheapest decisive test after the paste, no new owner step: the 18:41 resume pattern repeats on its own; or I read the next session turn read-only.

## BLOCKS?
- **Delegator wake / h5 ack: BLOCKED** until the Go slot holds a working key: any delegator turn (the pull and ack are tool calls inside a model turn) 403s, as turn 59 did.
- **G40: BLOCKED twice over:** the delegator cannot ack h5 or host Team E, AND the fleet shares the same store (`DSH_HOME` of the fleet unit is the same home, hot-reload on), so the Builder lane G40 needs, and any other lane start (rehearsal, order o9), would get the same 403. The fleet's lead and builder:1 are idle and have made no model call since 17:32, so no fleet failure is on record yet.
- **NOT blocked:** PA-9 and L4-A (Claude workflows), the board, the bus, the design work.
