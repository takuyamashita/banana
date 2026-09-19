import { TransportProvider, useQuery } from "@connectrpc/connect-query";
import { createApiTransport, StaffService } from "@platform/api-client";
import { Alert, Button } from "@platform/ui";
import { QueryClientProvider } from "@tanstack/react-query";
import type { User } from "oidc-client-ts";
import { useCallback, useEffect, useMemo, useState } from "react";

import { PayslipAdmin } from "./features/payroll/PayslipAdmin";
import { MyPayslips } from "./features/payroll/MyPayslips";
import { ProjectPanel } from "./features/project/ProjectPanel";
import { StaffPanel } from "./features/staff/StaffPanel";
import { createUserManager, restoreSession, SignInError, signOut } from "./lib/auth";
import type { RuntimeConfig } from "./lib/config";
import { errorMessage, SESSION_EXPIRED } from "./lib/errors";
import { createQueryClient } from "./lib/query";

export function App({ config }: { config: RuntimeConfig }) {
  const manager = useMemo(() => createUserManager(config), [config]);
  // undefined はログインしているか確かめている途中
  const [user, setUser] = useState<User | null | undefined>(undefined);
  const [notice, setNotice] = useState<string | null>(null);

  // トークンが切れた・更新できなかった・API に通らなくなったら、ログイン画面に戻してログインし直してもらう
  const endSession = useCallback(
    (message: string) => {
      void manager.removeUser();
      setNotice(message);
      setUser(null);
    },
    [manager],
  );

  const queryClient = useMemo(() => createQueryClient(() => endSession(SESSION_EXPIRED)), [endSession]);
  const transport = useMemo(
    () =>
      createApiTransport({
        baseUrl: config.apiBaseUrl,
        getAccessToken: async () => (await manager.getUser())?.access_token,
      }),
    [config, manager],
  );

  useEffect(() => {
    restoreSession(manager)
      .then(setUser)
      .catch((err: unknown) => {
        setNotice(err instanceof SignInError ? err.message : errorMessage(err));
        setUser(null);
      });
  }, [manager]);

  useEffect(() => {
    const expired = () => endSession(SESSION_EXPIRED);
    manager.events.addAccessTokenExpired(expired);
    manager.events.addSilentRenewError(expired);
    return () => {
      manager.events.removeAccessTokenExpired(expired);
      manager.events.removeSilentRenewError(expired);
    };
  }, [manager, endSession]);

  // ログイン画面に戻ったら、前のユーザーで取ったデータを残さない
  useEffect(() => {
    if (user === null) queryClient.clear();
  }, [user, queryClient]);

  if (user === undefined) {
    return (
      <main className="layout">
        <output>読み込み中…</output>
      </main>
    );
  }

  if (user === null) {
    return (
      <main className="layout">
        <h1>給与管理</h1>
        {notice && <Alert>{notice}</Alert>}
        <Button onClick={() => void manager.signinRedirect()}>ログイン</Button>
      </main>
    );
  }

  return (
    <TransportProvider transport={transport}>
      <QueryClientProvider client={queryClient}>
        <Home email={user.profile.email ?? ""} onSignOut={() => signOut(manager, config)} />
      </QueryClientProvider>
    </TransportProvider>
  );
}

type AdminTab = "payslips" | "staff" | "project";

const adminTabs = [
  ["payslips", "給与明細"],
  ["staff", "派遣社員"],
  ["project", "案件"],
] as const satisfies readonly (readonly [AdminTab, string])[];

/// ログインした人の画面。管理者なら管理メニュー、派遣社員なら自分の給与明細
function Home({ email, onSignOut }: { email: string; onSignOut: () => Promise<void> }) {
  const me = useQuery(StaffService.method.getMe, {});
  const [tab, setTab] = useState<AdminTab>("payslips");
  const [error, setError] = useState<string | null>(null);
  const isAdmin = me.data?.roles.includes("admin") ?? false;

  return (
    <main className="layout">
      <header className="header">
        <h1>給与管理</h1>
        <span className="who">{email}</span>
        <Button
          variant="secondary"
          onClick={() => void onSignOut().catch((err: unknown) => setError(errorMessage(err)))}
        >
          ログアウト
        </Button>
      </header>
      {error && <Alert>{error}</Alert>}
      {me.isPending && <output>読み込み中…</output>}
      {me.error && <Alert>{errorMessage(me.error)}</Alert>}

      {me.data && isAdmin && (
        <>
          <nav className="tabs" aria-label="管理メニュー">
            {adminTabs.map(([key, label]) => (
              <Button
                key={key}
                variant={tab === key ? "primary" : "secondary"}
                aria-pressed={tab === key}
                onClick={() => setTab(key)}
              >
                {label}
              </Button>
            ))}
          </nav>
          {tab === "payslips" && <PayslipAdmin />}
          {tab === "staff" && <StaffPanel />}
          {tab === "project" && <ProjectPanel />}
        </>
      )}

      {me.data && !isAdmin && me.data.staff && <MyPayslips staffId={me.data.staff.staffId} />}
      {me.data && !isAdmin && !me.data.staff && (
        <Alert>派遣社員として登録されていません。管理者に連絡してください。</Alert>
      )}
    </main>
  );
}
