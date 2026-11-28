// Multi-user sign-in & first-run administrator setup screen for `/` and `/phone`.
// Uses Ergon's house-frontend tokens; Light/Dark theme stays strictly in Settings (#67).
import { useState } from "react";
import { KeyRound, ShieldCheck, Smartphone } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Pill } from "@/components/kibo-ui/pill";
import { ErgonMark } from "@/shell/ErgonLogo";
import { loginLocal, setupInitialAdmin, type AuthMePayload } from "@/lib/api";

export function AuthScreen({
  auth,
  onAuthenticated,
}: {
  auth: AuthMePayload;
  onAuthenticated: () => Promise<void> | void;
}) {
  const isPhone = typeof window !== "undefined" && window.location.pathname.startsWith("/phone");
  const isSetup = auth.needs_setup;

  const [username, setUsername] = useState(isSetup ? "admin" : "");
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");

  const handleSetup = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr("");
    if (password.length < 8) {
      setErr("Password must be at least 8 characters.");
      return;
    }
    if (password !== confirmPassword) {
      setErr("Passwords do not match.");
      return;
    }
    setBusy(true);
    const res = await setupInitialAdmin({
      username: username.trim(),
      password,
      display_name: displayName.trim() || undefined,
      email: email.trim() || undefined,
    });
    setBusy(false);
    if (!res.ok) {
      setErr(res.error || "Could not create administrator account.");
      return;
    }
    await onAuthenticated();
  };

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr("");
    if (!username.trim() || !password) {
      setErr("Enter both username and password.");
      return;
    }
    setBusy(true);
    const res = await loginLocal({
      username: username.trim(),
      password,
    });
    setBusy(false);
    if (!res.ok) {
      setErr(res.error || "Invalid username or password.");
      return;
    }
    await onAuthenticated();
  };

  const oidcUrl = `/api/auth/oidc/login?next=${encodeURIComponent(
    typeof window !== "undefined" ? window.location.pathname + window.location.search : "/"
  )}`;

  return (
    <main
      id="auth-screen"
      data-auth-mode={isSetup ? "setup" : "login"}
      className="flex min-h-dvh w-full flex-col items-center justify-center bg-canvas px-4 py-8 text-ink"
    >
      <div className="w-full max-w-md rounded-card border border-hairline bg-surface p-6 shadow-card sm:p-7">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2.5">
            <span
              aria-hidden
              className="grid size-9 place-items-center rounded-sm border border-hairline bg-board text-ink"
            >
              <ErgonMark className="size-5 text-ink" />
            </span>
            <div>
              <h1 className="text-base font-semibold tracking-tight text-ink">
                {isSetup ? "Set up Ergon Admin" : "Sign in to Ergon"}
              </h1>
              <p className="text-xs text-ink-2">
                {isSetup
                  ? "Create the initial administrator account to secure this server."
                  : "Sign in with your account to access your isolated workspace."}
              </p>
            </div>
          </div>
        </div>

        <div className="mt-4 flex flex-wrap items-center gap-1.5">
          {auth.tls && (
            <Pill
              variant="secondary"
              className="border border-hairline bg-pill-active-bg px-2 py-0.5 text-xs font-medium text-pill-active-fg"
            >
              <ShieldCheck className="size-3" strokeWidth={2} aria-hidden />
              HTTPS / TLS
            </Pill>
          )}
          {isPhone && (
            <Pill
              variant="secondary"
              className="border border-hairline bg-board px-2 py-0.5 text-xs font-medium text-ink-2"
            >
              <Smartphone className="size-3" strokeWidth={1.75} aria-hidden />
              Field / Phone View
            </Pill>
          )}
          <Pill
            variant="secondary"
            className="border border-hairline bg-board px-2 py-0.5 text-xs text-ink-2"
          >
            Role-isolated workspaces
          </Pill>
        </div>

        {isSetup ? (
          <form
            id="auth-setup-form"
            onSubmit={handleSetup}
            className="mt-5 flex flex-col gap-3.5"
          >
            <div className="flex flex-col gap-1">
              <label htmlFor="auth-setup-username" className="text-xs font-medium text-ink">
                Admin username
              </label>
              <Input
                id="auth-setup-username"
                name="username"
                autoComplete="username"
                required
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="admin"
                className="min-h-11 bg-surface md:min-h-9"
              />
            </div>

            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
              <div className="flex flex-col gap-1">
                <label htmlFor="auth-setup-name" className="text-xs font-medium text-ink">
                  Full name
                </label>
                <Input
                  id="auth-setup-name"
                  name="display_name"
                  autoComplete="name"
                  value={displayName}
                  onChange={(e) => setDisplayName(e.target.value)}
                  placeholder="Alex Rivera"
                  className="min-h-11 bg-surface md:min-h-9"
                />
              </div>
              <div className="flex flex-col gap-1">
                <label htmlFor="auth-setup-email" className="text-xs font-medium text-ink">
                  Work email (optional)
                </label>
                <Input
                  id="auth-setup-email"
                  name="email"
                  type="email"
                  autoComplete="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="alex@company.com"
                  className="min-h-11 bg-surface md:min-h-9"
                />
              </div>
            </div>

            <div className="flex flex-col gap-1">
              <label htmlFor="auth-setup-password" className="text-xs font-medium text-ink">
                Password (min 8 characters)
              </label>
              <Input
                id="auth-setup-password"
                name="password"
                type="password"
                autoComplete="new-password"
                required
                minLength={8}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Strong passphrase"
                className="min-h-11 bg-surface md:min-h-9"
              />
            </div>

            <div className="flex flex-col gap-1">
              <label htmlFor="auth-setup-confirm" className="text-xs font-medium text-ink">
                Confirm password
              </label>
              <Input
                id="auth-setup-confirm"
                name="confirm_password"
                type="password"
                autoComplete="new-password"
                required
                minLength={8}
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder="Repeat passphrase"
                className="min-h-11 bg-surface md:min-h-9"
              />
            </div>

            {err && (
              <p
                id="auth-error"
                role="alert"
                className="rounded-sm border border-destructive/30 bg-status-stopped-bg px-3 py-2 text-xs text-status-stopped-fg"
              >
                {err}
              </p>
            )}

            <Button
              id="auth-setup-submit"
              type="submit"
              disabled={busy}
              className="mt-1 min-h-11 w-full md:min-h-9"
            >
              {busy ? "Creating administrator…" : "Create Administrator & Sign In"}
            </Button>
          </form>
        ) : (
          <form
            id="auth-login-form"
            onSubmit={handleLogin}
            className="mt-5 flex flex-col gap-3.5"
          >
            <div className="flex flex-col gap-1">
              <label htmlFor="auth-login-username" className="text-xs font-medium text-ink">
                Username or email
              </label>
              <Input
                id="auth-login-username"
                name="username"
                autoComplete="username"
                required
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="username or email"
                className="min-h-11 bg-surface md:min-h-9"
              />
            </div>

            <div className="flex flex-col gap-1">
              <label htmlFor="auth-login-password" className="text-xs font-medium text-ink">
                Password
              </label>
              <Input
                id="auth-login-password"
                name="password"
                type="password"
                autoComplete="current-password"
                required
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Enter your password"
                className="min-h-11 bg-surface md:min-h-9"
              />
            </div>

            {err && (
              <p
                id="auth-error"
                role="alert"
                className="rounded-sm border border-destructive/30 bg-status-stopped-bg px-3 py-2 text-xs text-status-stopped-fg"
              >
                {err}
              </p>
            )}

            <Button
              id="auth-login-submit"
              type="submit"
              disabled={busy}
              className="mt-1 min-h-11 w-full md:min-h-9"
            >
              {busy ? "Signing in…" : "Sign in"}
            </Button>

            {auth.oidc_enabled && (
              <>
                <div className="flex items-center gap-2 py-1 text-xs text-ink-3">
                  <span className="h-px flex-1 bg-hairline" />
                  <span>or</span>
                  <span className="h-px flex-1 bg-hairline" />
                </div>
                <Button
                  id="auth-oidc-btn"
                  type="button"
                  variant="outline"
                  className="min-h-11 w-full gap-2 md:min-h-9"
                  onClick={() => {
                    window.location.assign(oidcUrl);
                  }}
                >
                  <KeyRound className="size-3.5" strokeWidth={1.75} aria-hidden />
                  Sign in with SSO (OIDC)
                </Button>
              </>
            )}
          </form>
        )}

        <p className="mt-5 border-t border-hairline pt-3 text-xs leading-5 text-ink-3">
          Sessions use HttpOnly cookies with server-side expiry and role-scoped workspace isolation.
        </p>
      </div>
    </main>
  );
}
