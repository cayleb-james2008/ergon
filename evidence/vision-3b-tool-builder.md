# Vision Step 3b E2E Proof — Real Tool Builder

- Binary: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/target/debug/ergon`
- Model: `qwen2.5:3b` at `http://127.0.0.1:11434/v1`
- Scratch Data Dir: `/home/cayleb/Work/scratch/2026-11-27-ergon-full/e2e-data-3b`

## Verified Checks
1. Compile errors (`400`) and failing sandbox test assertions (`400`) rejected.
2. Valid tool (`Word Counter`) staged as `pending_approval` with `2/2` passing sandbox tests; uncallable (`404`) prior to approval.
3. Approving `tool_offer` decision installed `word-counter` for Employee A and published to shared Company Library.
4. Employee B installed `word-counter` from Company Library and executed it in a live Ollama `qwen2.5:3b` agent run (`words=5`).
5. Denying a second `tool_offer` kept it out of `library.json` and `suite.json`.

## Log Excerpt
```text
Compile error rejected -> 400
Failing test rejected -> 400
Staged tool package tp-word-counter-1795788911702-0 -> decision d1795788911703-0 (2/2 sandbox tests passed)
Unapproved tool run refused -> 404
Approved decision d1795788911703-0 -> Approved and installed "Word Counter" into your suite and the company library.
Employee A ran word-counter -> words=5
Employee B live Ollama qwen2.5:3b run using word-counter -> 200: [{"content": "Call the word-counter tool with {\"path\": \"bob_notes.txt\"}, then state the exact words=... output in your summary.", "role": "user"}, {"content": "Tool: word-counter\nwords=5", "role": "step"}, {"content": "The word-counter tool processed 'bob_notes.txt' and returned 5 words.", "role": "assistant"}, {"content": "Note: this run wrote no files; the summary also names bob_notes.txt, which this run did not write.", "role": "step"}]
Denied decision d1795788919439-1 -> never published to library
```
