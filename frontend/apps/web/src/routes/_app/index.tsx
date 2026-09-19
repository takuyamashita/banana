import { createFileRoute, redirect } from "@tanstack/react-router";

import { isAdmin } from "../../lib/roles";

/// 最初の画面。管理者は給与明細、派遣社員は自分の給与明細
export const Route = createFileRoute("/_app/")({
  beforeLoad: ({ context }) => {
    throw redirect({ to: isAdmin(context.me) ? "/payslips" : "/me" });
  },
});
