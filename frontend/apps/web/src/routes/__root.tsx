import { createRootRouteWithContext, Outlet, redirect } from "@tanstack/react-router";

import { NotFound } from "../layout/RouteStatus";
import type { RouterContext } from "../lib/context";

/// どの画面を開くときも、先にログインの状態を確かめる。ログインしていなければログイン画面へ
/// (開こうとしていた画面を戻り先として持っていく)
export const Route = createRootRouteWithContext<RouterContext>()({
  beforeLoad: async ({ context, location }) => {
    const user = await context.auth.restore();
    const onLogin = location.pathname === "/login";
    if (!user && !onLogin) throw redirect({ to: "/login", search: { returnTo: location.href } });
    if (user && onLogin) throw redirect({ to: "/" });
  },
  component: Outlet,
  notFoundComponent: NotFound,
});
