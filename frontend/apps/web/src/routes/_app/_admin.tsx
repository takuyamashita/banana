import { createFileRoute, redirect } from "@tanstack/react-router";

import { isAdmin } from "../../lib/roles";

/// 管理者だけの画面。管理者でなければ自分の給与明細へ(API も管理者以外を断る)
export const Route = createFileRoute("/_app/_admin")({
  beforeLoad: ({ context }) => {
    if (!isAdmin(context.me)) throw redirect({ to: "/me" });
  },
});
