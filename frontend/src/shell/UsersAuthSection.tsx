// Account session controls, admin user management (RBAC), and optional OIDC
// SSO settings embedded inside SettingsPanel.
import { useCallback, useEffect, useState } from "react";
import { KeyRound, LogOut, Shield, Trash2, UserPlus, Users } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Pill } from "@/components/kibo-ui/pill";
import {
  changeOwnPassword,
  createAuthUser,
  deleteAuthUser,
  fetchOidcConfig,
  listAuthUsers,
  saveOidcConfig,
  setupInitialAdmin,
  updateAuthUser,
  type AuthMePayload,
  type AuthRole,
  type AuthUserRecord,
} from "@/lib/api";

const ROLE_OPTIONS: { value: AuthRole; label: string }[] = [
  { value: "admin", label: "Admin" },
  { value: "pm", label: "Project Manager (pm)" },
  { value: "ae", label: "Account Executive (ae)" },
  { value: "office", label: "Office / Finance (office)" },
  { value: "installer", label: "Field Installer (installer)" },
  { value: "member", label: "Member (member)" },
];

export function UsersAuthSection({
  auth,
  onSignOut,
  onRefreshAuth,
  onUsersChanged,
}: {
  auth: AuthMePayload | null;
  onSignOut: () => Promise<void> | void;
  onRefreshAuth: () => Promise<void> | void;
  onUsersChanged?: () => void;
}) {
  const isAdmin = Boolean(auth?.user?.is_admin);
  const [users, setUsers] = useState<AuthUserRecord[]>([]);
  const [loadingUsers, setLoadingUsers] = useState(false);

  // Create user form (admin)
  const [newUsername, setNewUsername] = useState("");
  const [newName, setNewName] = useState("");
  const [newEmail, setNewEmail] = useState("");
  const [newRole, setNewRole] = useState<AuthRole>("pm");
  const [newPassword, setNewPassword] = useState("");
  const [creating, setCreating] = useState(false);
  const [userErr, setUserErr] = useState("");

  // Change own password
  const [pwOpen, setPwOpen] = useState(false);
  const [curPw, setCurPw] = useState("");
  const [nextPw, setNextPw] = useState("");
  const [pwSaving, setPwSaving] = useState(false);

  // Enable multi-user auth from single-user loopback mode
  const [setupOpen, setSetupOpen] = useState(false);
  const [setupUser, setSetupUser] = useState("admin");
  const [setupPass, setSetupPass] = useState("");
  const [setupSaving, setSetupSaving] = useState(false);

  // OIDC config (admin)
  const [oidcOpen, setOidcOpen] = useState(false);
  const [oidcEnabled, setOidcEnabled] = useState(false);
  const [oidcIssuer, setOidcIssuer] = useState("");
  const [oidcClientId, setOidcClientId] = useState("");
  const [oidcClientSecret, setOidcClientSecret] = useState("");
  const [oidcHasSecret, setOidcHasSecret] = useState(false);
  const [oidcRedirectUri, setOidcRedirectUri] = useState("");
  const [oidcDefaultRole, setOidcDefaultRole] = useState<AuthRole>("member");
  const [oidcAutoProvision, setOidcAutoProvision] = useState(true);
  const [oidcSaving, setOidcSaving] = useState(false);

  const loadUsers = useCallback(async () => {
    if (!isAdmin) return;
    setLoadingUsers(true);
    const r = await listAuthUsers();
    setLoadingUsers(false);
    if (r.ok) {
      setUsers(r.data.users);
    }
  }, [isAdmin]);

  const loadOidc = useCallback(async () => {
    if (!isAdmin) return;
    const r = await fetchOidcConfig();
    if (r.ok) {
      setOidcEnabled(r.data.enabled);
      setOidcIssuer(r.data.issuer || "");
      setOidcClientId(r.data.client_id || "");
      setOidcHasSecret(r.data.has_client_secret);
      setOidcRedirectUri(r.data.redirect_uri || "");
      setOidcDefaultRole(r.data.default_role || "member");
      setOidcAutoProvision(r.data.auto_provision);
    }
  }, [isAdmin]);

  useEffect(() => {
    if (isAdmin) {
      void loadUsers();
      void loadOidc();
    }
  }, [isAdmin, loadUsers, loadOidc]);

  const handleCreateUser = async (e: React.FormEvent) => {
    e.preventDefault();
    setUserErr("");
    if (!newUsername.trim() || newPassword.length < 8) {
      setUserErr("Enter a username and a password of at least 8 characters.");
      return;
    }
    setCreating(true);
    const r = await createAuthUser({
      username: newUsername.trim(),
      password: newPassword,
      role: newRole,
      display_name: newName.trim() || undefined,
      email: newEmail.trim() || undefined,
    });
    setCreating(false);
    if (!r.ok) {
      setUserErr(r.error || "Could not create user account.");
      return;
    }
    setNewUsername("");
    setNewName("");
    setNewEmail("");
    setNewPassword("");
    toast.success(`Created account "${r.data.user.username}" (${r.data.user.role}).`);
    await loadUsers();
    onUsersChanged?.();
  };

  const handleRoleChange = async (user: AuthUserRecord, role: AuthRole) => {
    const r = await updateAuthUser(user.id, { role });
    if (!r.ok) {
      toast.error(r.error || "Could not update role.");
      return;
    }
    toast.success(`Updated ${user.username} role to ${role}.`);
    await loadUsers();
    onUsersChanged?.();
  };

  const handleDeleteUser = async (user: AuthUserRecord) => {
    const r = await deleteAuthUser(user.id);
    if (!r.ok) {
      toast.error(r.error || "Could not delete user.");
      return;
    }
    toast.success(`Deleted account "${user.username}".`);
    await loadUsers();
    onUsersChanged?.();
  };

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    if (nextPw.length < 8) {
      toast.error("New password must be at least 8 characters.");
      return;
    }
    setPwSaving(true);
    const r = await changeOwnPassword({ current_password: curPw, new_password: nextPw });
    setPwSaving(false);
    if (!r.ok) {
      toast.error(r.error || "Could not change password.");
      return;
    }
    setCurPw("");
    setNextPw("");
    setPwOpen(false);
    toast.success("Password updated.");
  };

  const handleSaveOidc = async (e: React.FormEvent) => {
    e.preventDefault();
    setOidcSaving(true);
    const r = await saveOidcConfig({
      enabled: oidcEnabled,
      issuer: oidcIssuer.trim(),
      client_id: oidcClientId.trim(),
      client_secret: oidcClientSecret.trim() || undefined,
      redirect_uri: oidcRedirectUri.trim(),
      default_role: oidcDefaultRole,
      auto_provision: oidcAutoProvision,
    });
    setOidcSaving(false);
    if (!r.ok) {
      toast.error(r.error || "Could not save OIDC settings.");
      return;
    }
    setOidcClientSecret("");
    setOidcHasSecret(r.data.has_client_secret);
    toast.success("OIDC SSO settings saved.");
    await onRefreshAuth();
  };

  const handleEnableMultiUser = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!setupUser.trim() || setupPass.length < 8) {
      toast.error("Enter an admin username and an 8+ character password.");
      return;
    }
    setSetupSaving(true);
    const r = await setupInitialAdmin({
      username: setupUser.trim(),
      password: setupPass,
    });
    setSetupSaving(false);
    if (!r.ok) {
      toast.error(r.error || "Could not create initial admin account.");
      return;
    }
    setSetupPass("");
    setSetupOpen(false);
    toast.success("Multi-user authentication enabled.");
    await onRefreshAuth();
  };

  if (!auth) return null;

  return (
    <section
      id="settings-auth-section"
      aria-labelledby="settings-auth-h"
      className="flex flex-col gap-3 rounded-card border border-hairline bg-board/40 p-3.5"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <Shield className="size-4 text-icon" strokeWidth={1.5} aria-hidden />
          <h3 id="settings-auth-h" className="text-sm font-medium text-ink">
            Account & Access Control
          </h3>
        </div>
        <div className="flex items-center gap-1.5">
          {auth.tls && (
            <Pill variant="secondary" className="bg-pill-active-bg px-2 py-0.5 text-xs font-medium text-pill-active-fg">
              HTTPS / TLS
            </Pill>
          )}
          {auth.user && (
            <Pill variant="secondary" className="border border-hairline bg-surface px-2 py-0.5 text-xs text-ink">
              {auth.user.role}
            </Pill>
          )}
        </div>
      </div>

      {auth.authenticated && auth.user ? (
        <div className="flex flex-col gap-2.5 rounded-sm border border-hairline bg-surface p-3">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div>
              <p className="text-sm font-medium text-ink">{auth.user.display_name}</p>
              <p className="mono text-xs text-ink-2">
                @{auth.user.username}
                {auth.user.email ? ` · ${auth.user.email}` : ""}
              </p>
            </div>
            <div className="flex items-center gap-1.5">
              {!auth.user.break_glass && (
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  className="max-md:min-h-11"
                  onClick={() => setPwOpen((v) => !v)}
                >
                  Change password
                </Button>
              )}
              <Button
                id="settings-sign-out-btn"
                type="button"
                size="xs"
                variant="outline"
                className="max-md:min-h-11 gap-1"
                onClick={() => void onSignOut()}
              >
                <LogOut className="size-3.5" strokeWidth={1.5} aria-hidden />
                Sign out
              </Button>
            </div>
          </div>

          {pwOpen && (
            <form onSubmit={handleChangePassword} className="mt-1 flex flex-col gap-2 border-t border-hairline pt-2.5">
              <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                <Input
                  type="password"
                  placeholder="Current password"
                  autoComplete="current-password"
                  value={curPw}
                  onChange={(e) => setCurPw(e.target.value)}
                  required
                />
                <Input
                  type="password"
                  placeholder="New password (8+ chars)"
                  autoComplete="new-password"
                  value={nextPw}
                  onChange={(e) => setNextPw(e.target.value)}
                  required
                  minLength={8}
                />
              </div>
              <div className="flex justify-end gap-2">
                <Button type="button" size="xs" variant="ghost" onClick={() => setPwOpen(false)}>
                  Cancel
                </Button>
                <Button type="submit" size="xs" disabled={pwSaving}>
                  {pwSaving ? "Updating…" : "Update password"}
                </Button>
              </div>
            </form>
          )}
        </div>
      ) : auth.needs_setup ? (
        <div className="flex flex-col gap-2 rounded-sm border border-hairline bg-surface p-3">
          <p className="text-xs leading-5 text-ink-2">
            Running in local single-user mode. Create an administrator account to enforce login and role-based workspace isolation.
          </p>
          {!setupOpen ? (
            <div>
              <Button type="button" size="xs" variant="outline" onClick={() => setSetupOpen(true)}>
                <UserPlus className="size-3.5" strokeWidth={1.5} aria-hidden />
                Enable multi-user login
              </Button>
            </div>
          ) : (
            <form onSubmit={handleEnableMultiUser} className="flex flex-col gap-2">
              <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                <Input
                  value={setupUser}
                  onChange={(e) => setSetupUser(e.target.value)}
                  placeholder="Admin username"
                  autoComplete="username"
                  spellCheck={false}
                  required
                />
                <Input
                  type="password"
                  value={setupPass}
                  onChange={(e) => setSetupPass(e.target.value)}
                  placeholder="Password (8+ chars)"
                  autoComplete="new-password"
                  required
                  minLength={8}
                />
              </div>
              <div className="flex justify-end gap-2">
                <Button type="button" size="xs" variant="ghost" onClick={() => setSetupOpen(false)}>
                  Cancel
                </Button>
                <Button type="submit" size="xs" disabled={setupSaving}>
                  {setupSaving ? "Enabling…" : "Create admin"}
                </Button>
              </div>
            </form>
          )}
        </div>
      ) : null}

      {isAdmin && (
        <div className="flex flex-col gap-3 border-t border-hairline pt-3">
          <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-1.5">
              <Users className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
              <span className="text-xs font-medium text-ink">
                Team Accounts & Roles ({users.length})
              </span>
            </div>
            {loadingUsers && <span className="mono text-xs text-ink-3">Loading…</span>}
          </div>

          <ul id="admin-users-list" className="flex flex-col gap-1.5">
            {users.map((u) => (
              <li
                key={u.id}
                data-user-id={u.id}
                data-username={u.username}
                className="flex flex-wrap items-center justify-between gap-2 rounded-sm border border-hairline bg-surface px-2.5 py-2"
              >
                <div className="min-w-0 flex-1">
                  <p className="truncate text-xs font-medium text-ink">
                    {u.display_name}{" "}
                    <span className="mono font-normal text-ink-2">@{u.username}</span>
                  </p>
                  <p className="truncate text-xs text-ink-3">
                    workspace: <span className="mono">{u.person_id}</span>
                    {u.oidc_linked ? " · OIDC" : ""}
                  </p>
                </div>
                <div className="flex items-center gap-1.5">
                  <select
                    aria-label={`Role for ${u.username}`}
                    value={u.role}
                    onChange={(e) => void handleRoleChange(u, e.target.value as AuthRole)}
                    className="h-8 rounded-sm border border-hairline bg-board px-2 text-xs text-ink"
                  >
                    {ROLE_OPTIONS.map((opt) => (
                      <option key={opt.value} value={opt.value}>
                        {opt.value}
                      </option>
                    ))}
                  </select>
                  <Button
                    type="button"
                    size="icon-xs"
                    variant="ghost"
                    aria-label={`Delete ${u.username}`}
                    onClick={() => void handleDeleteUser(u)}
                  >
                    <Trash2 className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                  </Button>
                </div>
              </li>
            ))}
          </ul>

          <form
            id="admin-create-user-form"
            onSubmit={handleCreateUser}
            className="flex flex-col gap-2 rounded-sm border border-hairline bg-surface p-2.5"
          >
            <span className="text-xs font-medium text-ink">Add user account</span>
            <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
              <Input
                id="admin-new-username"
                placeholder="Username (e.g. maya)"
                value={newUsername}
                onChange={(e) => setNewUsername(e.target.value)}
                required
              />
              <Input
                id="admin-new-name"
                placeholder="Full name (e.g. Maya Chen)"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
              />
              <Input
                id="admin-new-email"
                type="email"
                placeholder="Email (optional)"
                value={newEmail}
                onChange={(e) => setNewEmail(e.target.value)}
              />
              <select
                id="admin-new-role"
                aria-label="New user role"
                value={newRole}
                onChange={(e) => setNewRole(e.target.value as AuthRole)}
                className="h-9 rounded-sm border border-input bg-surface px-2.5 text-sm text-ink"
              >
                {ROLE_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {opt.label}
                  </option>
                ))}
              </select>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Input
                id="admin-new-password"
                type="password"
                placeholder="Initial password (8+ chars)"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                required
                minLength={8}
                className="min-w-[180px] flex-1"
              />
              <Button id="admin-create-user-submit" type="submit" size="sm" disabled={creating}>
                <UserPlus className="size-3.5" strokeWidth={1.5} aria-hidden />
                {creating ? "Creating…" : "Create user"}
              </Button>
            </div>
            {userErr && (
              <p role="alert" className="text-xs text-destructive-foreground">
                {userErr}
              </p>
            )}
          </form>

          {/* Optional OIDC SSO Configuration */}
          <div className="rounded-sm border border-hairline bg-surface p-2.5">
            <button
              type="button"
              onClick={() => setOidcOpen((v) => !v)}
              className="flex w-full items-center justify-between text-left text-xs font-medium text-ink"
            >
              <span className="flex items-center gap-1.5">
                <KeyRound className="size-3.5 text-icon" strokeWidth={1.5} aria-hidden />
                OIDC Single Sign-On (Optional)
              </span>
              <span className="mono text-ink-3">{oidcEnabled ? "Enabled" : "Off"}</span>
            </button>
            {oidcOpen && (
              <form onSubmit={handleSaveOidc} className="mt-2.5 flex flex-col gap-2 border-t border-hairline pt-2.5">
                <label className="flex items-center gap-2 text-xs text-ink">
                  <input
                    type="checkbox"
                    checked={oidcEnabled}
                    onChange={(e) => setOidcEnabled(e.target.checked)}
                  />
                  Enable OIDC login on the sign-in screen
                </label>
                <Input
                  placeholder="Issuer URL (https://accounts.google.com)"
                  value={oidcIssuer}
                  onChange={(e) => setOidcIssuer(e.target.value)}
                />
                <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                  <Input
                    placeholder="Client ID"
                    value={oidcClientId}
                    onChange={(e) => setOidcClientId(e.target.value)}
                  />
                  <Input
                    type="password"
                    placeholder={oidcHasSecret ? "Client secret (saved — leave blank to keep)" : "Client secret (optional for PKCE)"}
                    value={oidcClientSecret}
                    onChange={(e) => setOidcClientSecret(e.target.value)}
                  />
                </div>
                <Input
                  placeholder="Redirect URI (blank = auto /api/auth/oidc/callback)"
                  value={oidcRedirectUri}
                  onChange={(e) => setOidcRedirectUri(e.target.value)}
                />
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <label className="flex items-center gap-2 text-xs text-ink">
                    <input
                      type="checkbox"
                      checked={oidcAutoProvision}
                      onChange={(e) => setOidcAutoProvision(e.target.checked)}
                    />
                    Auto-provision new users as
                  </label>
                  <select
                    aria-label="OIDC default role"
                    value={oidcDefaultRole}
                    onChange={(e) => setOidcDefaultRole(e.target.value as AuthRole)}
                    className="h-8 rounded-sm border border-hairline bg-board px-2 text-xs text-ink"
                  >
                    {ROLE_OPTIONS.map((opt) => (
                      <option key={opt.value} value={opt.value}>
                        {opt.value}
                      </option>
                    ))}
                  </select>
                </div>
                <div className="flex justify-end">
                  <Button type="submit" size="xs" disabled={oidcSaving}>
                    {oidcSaving ? "Saving…" : "Save OIDC config"}
                  </Button>
                </div>
              </form>
            )}
          </div>
        </div>
      )}
    </section>
  );
}
