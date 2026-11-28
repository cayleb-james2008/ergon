# Self-Hosting Ergon on Coolify

Run Ergon for your entire team on an office mini-PC or a \$5–\$10/month VPS with zero per-seat markup—your only variable cost is API credits (such as **OpenCode Go** at **\$20/employee** defaulting to **`deepseek-v4.1-flash`**, or your own Anthropic / OpenAI / local Ollama endpoint).

## Why Coolify + Ergon Fits

- **Zero external databases:** One ~12 MiB binary + a single `/data` volume holds encrypted credentials (`secrets.enc.json`), workspace sandboxes, routines, and receipts.
- **Automatic HTTPS for `/phone` & OAuth2:** Coolify's Traefik proxy provisions Let's Encrypt TLS automatically, and `ERGON_TRUST_PROXY=1` + `ERGON_PUBLIC_URL` ensure `Secure` session cookies and `https://` Google Workspace / Microsoft 365 OAuth callback URLs work out of the box.
- **Shared key OR per-employee seats:** Configure a single company API key in Coolify, or let each employee enter their own **\$20/mo OpenCode Go seat key** in **Settings → Model connection → Personal AI seat key** (encrypted per person in `secrets.enc.json`, never in plaintext).

---

## Option A — 2-Minute Deploy Using Prebuilt Image (Recommended)

1. In Coolify, click **+ New Resource → Docker Compose** and paste [docker-compose.coolify.yml](../docker-compose.coolify.yml).
2. Under **Domains**, assign your hostname (e.g. `https://ergon.yourdomain.com`) mapped to port `8787`.
3. Under **Environment Variables**, set:
   - `ERGON_MODEL_BASE_URL=https://opencode.ai/zen/go/v1` (default)
   - `ERGON_MODEL_ID=deepseek-v4.1-flash` (default)
   - `ERGON_MODEL_API_KEY=<your-opencode-go-or-provider-key>` *(optional if each employee sets their own Personal AI seat key in Settings)*
   - *(Optional initial admin)* `ERGON_ADMIN_USER=admin` and `ERGON_ADMIN_PASSWORD=<strong-password>`
4. Click **Deploy**. Coolify waits for `ergon healthcheck` (`GET /api/health`) and switches traffic to HTTPS.
5. Open your domain in a browser to complete initial admin setup (if not set via env vars) and add employees under **Settings → Users**.

## Option B — Build Directly from Git in Coolify

1. In Coolify, add a **Public/Private Repository** pointed at `https://github.com/cayleb-james2008/ergon`.
2. Choose **Build Pack: Dockerfile** (`Dockerfile` at repo root compiles `--release --locked` with embedded assets).
3. Set persistent storage volume mount `/data` and port `8787`, plus `ERGON_PUBLIC_URL=https://ergon.yourdomain.com`, `ERGON_TRUST_PROXY=1`, and your model environment variables.

---

## Per-Employee OpenCode Go Seats (`$20/employee`)

To isolate rate limits and track per-employee spend directly on their own plan:

1. Keep `ERGON_MODEL_BASE_URL=https://opencode.ai/zen/go/v1` and `ERGON_MODEL_ID=deepseek-v4.1-flash` as the workspace default (you can leave `ERGON_MODEL_API_KEY` empty or provide a fallback key).
2. Each employee opens **Settings → Model connection → Personal AI seat key**, clicks **OpenCode Go (DeepSeek v4.1 Flash)**, pastes their seat key, and clicks **Save personal key**.
3. Keys are encrypted at rest inside `<person_data_dir>/secrets.enc.json` (`0600`) and fall back cleanly to the company default if cleared.

---

## Google Workspace & Microsoft 365 OAuth2 Behind Coolify

Because Ergon uses `ERGON_PUBLIC_URL` (and `X-Forwarded-Proto: https` with `ERGON_TRUST_PROXY=1`):

1. Open **Connected apps → Setup sign-in** and copy the `Redirect URI` (`https://ergon.yourdomain.com/api/plugins/oauth/callback`).
2. Register that exact HTTPS URL in Google Cloud Console or Microsoft Entra ID.
3. Every employee can then click **Sign in with Google** or **Sign in with Microsoft** in one click from desktop or `/phone`.

---

## Backups & Upgrades

- **Persistent volume:** Everything lives under `/data`. Back up the Docker volume `ergon_data` or schedule built-in zip backups under **Settings → Backups & Recovery**.
- **Upgrades:** Click **Redeploy / Pull latest image** in Coolify to pull the newest `ghcr.io/cayleb-james2008/ergon:latest` release (`linux/amd64` and `linux/arm64`).
