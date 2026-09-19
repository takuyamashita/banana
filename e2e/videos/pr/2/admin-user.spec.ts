// 動作確認の動画の台本。管理者を追加し、その人が実際に管理者としてログインできるところまで見せる。
// 確かめるテスト(tests/)と同じく、画面が変わって操作が通らなくなれば録画も失敗する
import { LoginPage } from "../../../pages/login-page";
import { ADMIN, unique } from "../../../support/api";
import { caption, expect, test } from "../../../support/video";

test("管理者を追加し、その管理者がログインして管理の操作ができる", async ({ record }) => {
  const email = `${unique("admin")}@example.com`;
  const temporaryPassword = "Temp-pass-1";
  const newPassword = "New-password-1";

  const page = await record("追加する側の管理者");
  await new LoginPage(page).login(ADMIN.email, ADMIN.password);
  await caption(page, "① 管理者でログインする");

  await page.getByRole("link", { name: "管理者" }).click();
  await expect(page).toHaveURL(/\/admins$/);
  await caption(page, "② 管理メニューに「管理者」が増えた(/admins)");

  await page.getByLabel("メールアドレス").fill(email);
  await page.getByLabel("仮パスワード").fill(temporaryPassword);
  await caption(page, `③ メールアドレスと仮パスワードを入れる(${email})`, 3000);

  await page.getByRole("button", { name: "追加する" }).click();
  await expect(page.getByRole("status")).toContainText(`${email} を管理者にしました。`);
  await caption(page, "④ 誰を管理者にしたかが出て、入力欄は空に戻る", 3000);

  await page.getByLabel("メールアドレス").fill(email);
  await page.getByLabel("仮パスワード").fill(temporaryPassword);
  await page.getByRole("button", { name: "追加する" }).click();
  await expect(page.getByRole("alert")).toHaveText("同じメールアドレスの利用者が既にいます");
  await caption(page, "⑤ 同じメールアドレスでは作れない(サーバーの文言をそのまま出す)", 3000);

  // 付いたロールは画面からは見えないので、本人がログインできるところまでを見せる
  const added = await record("追加された管理者");
  await new LoginPage(added).login(email, temporaryPassword, newPassword);
  await caption(added, "⑥ 追加された管理者がログインする(初回はパスワードを変える)", 3000);

  await expect(added).toHaveURL(/\/payslips$/);
  await expect(added.getByRole("navigation", { name: "管理メニュー" })).toBeVisible();
  await caption(added, "⑦ 管理者の入口(給与明細)に入り、管理メニューが出る = admin ロールが付いている", 3500);

  await added.getByRole("link", { name: "派遣社員" }).click();
  await expect(added).toHaveURL(/\/staff$/);
  await expect(added.getByRole("table", { name: "登録済みの派遣社員" })).toBeVisible();
  await caption(added, "⑧ 管理者だけの画面(派遣社員の一覧)も開ける", 3000);
});
