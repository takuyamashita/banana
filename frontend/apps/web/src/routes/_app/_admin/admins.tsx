import { createFileRoute } from "@tanstack/react-router";

import { AdminUserPage } from "../../../features/admin-user/AdminUserPanel";

// 追加するだけの画面なので、先に取っておくデータはない(loader を置かない)
export const Route = createFileRoute("/_app/_admin/admins")({
  component: AdminUserPage,
});
