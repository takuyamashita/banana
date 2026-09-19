import type { GetMeResponse } from "@platform/api-client";

/// 管理者か(管理メニューを出し、管理の画面に入れる)
export function isAdmin(me: GetMeResponse): boolean {
  return me.roles.includes("admin");
}
