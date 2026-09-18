import { createApiClients, type GetMeResponse } from "@platform/api-client";
import { Alert, Button } from "@platform/ui";
import type { User } from "oidc-client-ts";
import { useEffect, useMemo, useState } from "react";

import { FinalizePayslipForm } from "./features/payroll/FinalizePayslipForm";
import { MyPayslips } from "./features/payroll/MyPayslips";
import { ProjectPanel } from "./features/project/ProjectPanel";
import { StaffPanel } from "./features/staff/StaffPanel";
import { ApiProvider, errorMessage } from "./lib/api";
import { createUserManager, restoreSession, signOut } from "./lib/auth";
import type { RuntimeConfig } from "./lib/config";

type AdminTab = "finalize" | "staff" | "project";

export function App({ config }: { config: RuntimeConfig }) {
  const manager = useMemo(() => createUserManager(config), [config]);
  const [user, setUser] = useState<User | null | undefined>(undefined);
  const [me, setMe] = useState<GetMeResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [tab, setTab] = useState<AdminTab>("finalize");

  const clients = useMemo(
    () =>
      createApiClients({
        baseUrl: config.apiBaseUrl,
        getAccessToken: async () => (await manager.getUser())?.access_token,
      }),
    [config, manager],
  );

  useEffect(() => {
    restoreSession(manager)
      .then(setUser)
      .catch((err: unknown) => {
        setError(errorMessage(err));
        setUser(null);
      });
  }, [manager]);

  useEffect(() => {
    if (!user) return;
    clients.staff
      .getMe({})
      .then(setMe)
      .catch((err: unknown) => setError(errorMessage(err)));
  }, [user, clients]);

  if (user === undefined) return <main className="layout">読み込み中…</main>;

  if (user === null) {
    return (
      <main className="layout">
        <h1>給与管理</h1>
        {error && <Alert>{error}</Alert>}
        <Button onClick={() => void manager.signinRedirect()}>ログイン</Button>
      </main>
    );
  }

  const isAdmin = me?.roles.includes("admin") ?? false;

  return (
    <ApiProvider clients={clients}>
      <main className="layout">
        <header className="header">
          <h1>給与管理</h1>
          <span className="who">{user.profile.email}</span>
          <Button variant="secondary" onClick={() => void signOut(manager)}>
            ログアウト
          </Button>
        </header>
        {error && <Alert>{error}</Alert>}

        {me && isAdmin && (
          <>
            <nav className="tabs" aria-label="管理メニュー">
              {(
                [
                  ["finalize", "給与確定"],
                  ["staff", "派遣社員"],
                  ["project", "案件"],
                ] as const
              ).map(([key, label]) => (
                <Button key={key} variant={tab === key ? "primary" : "secondary"} onClick={() => setTab(key)}>
                  {label}
                </Button>
              ))}
            </nav>
            {tab === "finalize" && <FinalizePayslipForm />}
            {tab === "staff" && <StaffPanel />}
            {tab === "project" && <ProjectPanel />}
          </>
        )}

        {me && !isAdmin && me.staff && <MyPayslips staffId={me.staff.staffId} />}
        {me && !isAdmin && !me.staff && <Alert>派遣社員として登録されていません。管理者に連絡してください。</Alert>}
      </main>
    </ApiProvider>
  );
}
