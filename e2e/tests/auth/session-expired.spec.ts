import { LoginPage } from "../../pages/login-page";
import { ADMIN } from "../../support/api";
import { expect, test } from "../../support/fixtures";

test("API にトークンが通らなくなったら、ログイン画面に戻してログインし直してもらう", async ({ page }) => {
  await new LoginPage(page).login(ADMIN.email, ADMIN.password);
  await expect(page.getByRole("button", { name: "ログアウト" })).toBeVisible();

  // トークンを失った状態を作る(期限切れ・失効と同じく、次の API 呼び出しが Unauthenticated になる)
  await page.evaluate(() => window.sessionStorage.clear());
  // まだ取っていないデータ(選んだ派遣社員の給与明細)を取りに行かせる
  await page.getByLabel("派遣社員").selectOption({ index: 1 });

  await expect(page.getByRole("alert")).toHaveText("ログインの有効期限が切れました。もう一度ログインしてください。");
  await expect(page.getByRole("button", { name: "ログイン" })).toBeVisible();
  // ログインし直したら、切れたときの画面に戻る
  await expect(page).toHaveURL(/\/login\?returnTo=%2Fpayslips%3FstaffId%3D\d+$/);
});
