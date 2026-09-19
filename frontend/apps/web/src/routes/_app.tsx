import { StaffService } from "@platform/api-client";
import { useMutation } from "@tanstack/react-query";
import { createFileRoute, Outlet, useMatches } from "@tanstack/react-router";
import { useSyncExternalStore } from "react";

import { AppLayout } from "../layout/AppLayout";
import { errorMessage } from "../lib/errors";
import { ensure } from "../lib/loaders";
import { isAdmin } from "../lib/roles";

/// ログインした人の画面の枠。自分が誰か(ロールと派遣社員の登録)を先に取り、下の画面に渡す
export const Route = createFileRoute("/_app")({
  beforeLoad: async ({ context, location }) => ({
    me: await ensure(context, StaffService.method.getMe, {}, location.href),
  }),
  component: AppRoute,
});

function AppRoute() {
  const { auth, me } = Route.useRouteContext();
  const { user } = useSyncExternalStore(auth.subscribe, auth.getSnapshot);
  const signOut = useMutation({ mutationFn: auth.signOut });
  // 開いている画面(いちばん内側のルート)が決めたシステム
  const system = useMatches({ select: (matches) => matches.findLast((m) => m.staticData.system)?.staticData.system });
  return (
    <AppLayout
      system={system ?? "payroll"}
      email={user?.profile.email ?? ""}
      menu={isAdmin(me) ? "admin" : me.staff ? "staff" : "none"}
      onSignOut={() => signOut.mutate()}
      signOutError={signOut.error ? errorMessage(signOut.error) : null}
    >
      <Outlet />
    </AppLayout>
  );
}
