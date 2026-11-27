# Live External Mailbox E2E Proof (Ethereal SMTP STARTTLS + IMAPS)

- **Date:** 2026-11-27
- **Binary:** `/home/cayleb/Work/scratch/2026-11-27-ergon-full/target/debug/ergon` (`v0.40.0`)
- **Server:** `127.0.0.1:18112` (scratch `ERGON_DATA_DIR`, deleted after verification)
- **External Mail Provider:** Ephemeral Ethereal inbox (`POST https://api.nodemailer.com/user`)
  - Outbound: `smtp.ethereal.email:587` (`starttls`)
  - Inbound: `imap.ethereal.email:993` (`imaps`, `INBOX`, `mark_seen: true`, `subject_filter: "Ergon Live Proof"`)
- **Raw Scrubbed Log:** [ethereal-smtp-imap-live.log](file:///home/cayleb/Work/projects/ergon/evidence/existing/ethereal-smtp-imap-live.log)

## Verification Steps & Observed Results

1. **Workspace & Linked Routine Seed:**
   - Created `reports/ethereal-probe.csv` (`62` bytes) via `PUT /api/files/content`.
   - Created zero-LLM linked routine `Ethereal IMAP Intake Routine` (`rtn-1795786301044-0`) via `POST /api/routines` with `write_document` writing `Ethereal Intake Note.md` from `{{inputs.trigger_file}}`.
2. **Connector & Secret Vault Hardening:**
   - Created `smtp_relay` outbound connector (`conn-1795786301053-0`, `require_approval: true`) via `POST /api/connectors/outbound`. Password omitted from HTTP response.
   - Created `imap_poll` trigger (`trg-1795786301063-1`, linked to `rtn-1795786301044-0`) via `POST /api/connectors/triggers`. Password omitted from HTTP response.
   - Verified `secrets.enc.json` permissions are `0o600` (`-rw-------`).
   - Recursive binary scan of the entire `DATA_DIR` found **0** plaintext occurrences of the Ethereal password.
3. **Human-Approved Outbound STARTTLS Delivery with Attachment:**
   - Triggered `POST /api/connectors/outbound/conn-1795786301053-0/test` with workspace attachment `reports/ethereal-probe.csv` -> HTTP `202` (`status: "held"`, `decision_id: "d1795786301074-0"`).
   - Approved `POST /api/decisions/d1795786301074-0/approve` -> HTTP `200`, `outcome: "Approved and delivered via \"Ethereal Live SMTP Relay\" (HTTP 250, 794 bytes)."`.
   - Delivery log (`GET /api/connectors`) recorded `http_status: 250`, `250 Accepted [STATUS=new MSGID=awmGPDsrpuHBWYn3awmGPnxXAa9Zb9owAAAAAawyjsNY5f38nznsVGTQYZE]`.
4. **Live IMAPS Intake & Routine Execution:**
   - Ergon's background `imap_poll` worker polled `imap.ethereal.email:993` over IMAPS, ingested `intake/imap-1795786306487-Ergon_Live_Proof_6b_STARTTLS.md`, extracted byte-exact attachment `intake/ethereal-probe.csv`, and triggered linked pipeline run `prun-1795786306604-0` (`status: "completed"`).
   - Verified `Ethereal Intake Note.md` cites `Trigger file: intake/imap-1795786306487-Ergon_Live_Proof_6b_STARTTLS.md`.
   - `GET /api/receipts/verify` returned `{"total": 1, "valid": true}`.
5. **Cleanup & Redaction:**
   - Server drained cleanly on `SIGTERM` (`rc=0`), all Ethereal email/user/password strings were scrubbed before writing logs, and the scratch `DATA_DIR` was deleted.
