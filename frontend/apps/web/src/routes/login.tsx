import { createFileRoute } from "@tanstack/react-router";
import { useSyncExternalStore } from "react";

import { LoginScreen } from "../layout/LoginScreen";
import { safeReturnTo } from "../lib/auth";

export const Route = createFileRoute("/login")({
  // 戻り先は同じサイトの中のパスだけ受け付ける
  validateSearch: (search): { returnTo?: string } => {
    const returnTo = safeReturnTo(search["returnTo"]);
    return returnTo === undefined ? {} : { returnTo };
  },
  component: LoginRoute,
});

function LoginRoute() {
  const { auth } = Route.useRouteContext();
  const { notice } = useSyncExternalStore(auth.subscribe, auth.getSnapshot);
  const { returnTo } = Route.useSearch();
  return <LoginScreen notice={notice} onSignIn={() => void auth.signIn(returnTo)} />;
}
