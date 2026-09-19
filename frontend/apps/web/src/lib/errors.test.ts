import { Code, ConnectError } from "@platform/api-client";

import { errorMessage, SESSION_EXPIRED } from "./errors";

test("利用者に向けたサーバーの文言はそのまま出す", () => {
  expect(errorMessage(new ConnectError("この月の給与明細は既にあります", Code.AlreadyExists))).toBe(
    "この月の給与明細は既にあります",
  );
  expect(errorMessage(new ConnectError("この操作は管理者だけができます", Code.PermissionDenied))).toBe(
    "この操作は管理者だけができます",
  );
});

test("ログインが切れたら、ログインし直すように伝える", () => {
  expect(errorMessage(new ConnectError("ログインしてください", Code.Unauthenticated))).toBe(SESSION_EXPIRED);
});

test("つながらない・内部の失敗・想定外は、原因の詳細を出さずに定型文にする", () => {
  expect(errorMessage(new ConnectError("upstream connect error", Code.Unavailable))).toMatch(/時間をおいて/);
  expect(errorMessage(new ConnectError("内部エラー", Code.Internal))).toMatch(/予期しないエラー/);
  expect(errorMessage(new TypeError("Failed to fetch"))).toMatch(/予期しないエラー/);
});
