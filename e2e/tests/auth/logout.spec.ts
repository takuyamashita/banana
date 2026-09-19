import { expect, test } from "@playwright/test";

import { LoginPage } from "../../pages/login-page";
import { ADMIN } from "../../support/api";

test("ログアウトすると認証基盤のログインも終わり、次にログインするときはパスワードを聞かれる", async ({ page }) => {
  await new LoginPage(page).login(ADMIN.email, ADMIN.password);

  await page.getByRole("button", { name: "ログアウト" }).click();
  await page.getByRole("button", { name: "ログイン" }).click();

  // 認証基盤にログインが残っていれば、パスワードを聞かれずに戻ってきてしまう
  await expect(page.locator("#password")).toBeVisible();
});
