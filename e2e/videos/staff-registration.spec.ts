// 動作確認の動画: 管理者が派遣社員を登録し、登録された人が初回ログインでパスワードを変えて使い始める
import { LoginPage } from "../pages/login-page";
import { ADMIN, unique } from "../support/api";
import { caption, expect, test } from "../support/video";

test("管理者が派遣社員を登録し、本人が初回ログインで使い始める", async ({ record }) => {
  const email = `${unique("staff")}@example.com`;
  const displayName = unique("派遣");
  const temporaryPassword = "Temp-pass-1";

  const admin = await record("管理者");
  await new LoginPage(admin).login(ADMIN.email, ADMIN.password);
  await admin.getByRole("link", { name: "派遣社員" }).click();
  await caption(admin, "① 管理者が派遣社員の画面を開く");

  await admin.getByLabel("メールアドレス").fill(email);
  await admin.getByLabel("表示名").fill(displayName);
  await admin.getByLabel("仮パスワード").fill(temporaryPassword);
  await caption(admin, "② メールアドレス・表示名・仮パスワードを入れる", 1500);
  await admin.getByRole("button", { name: "登録する" }).click();
  await expect(admin.getByRole("status")).toContainText("を登録しました");
  await caption(admin, "③ 登録できた。初回ログインでパスワードの変更を求めると知らせる", 3000);
  const row = admin.getByRole("row", { name: new RegExp(displayName) });
  await row.scrollIntoViewIfNeeded();
  await expect(row).toBeVisible();
  await caption(admin, "④ 登録済みの一覧に出る", 2500);

  const self = await record("本人");
  await self.goto("/");
  await caption(self, "⑤ 登録された本人が、仮パスワードでログインする", 2000);
  await new LoginPage(self).signIn(email, temporaryPassword, "New-pass-12345");
  await expect(self.getByRole("heading", { name: "自分の給与明細" })).toBeVisible();
  await caption(self, "⑥ 新しいパスワードに変えて使い始める(給与明細はまだない)", 3000);
});
