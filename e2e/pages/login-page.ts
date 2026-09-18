import type { Page } from "@playwright/test";

/// Keycloak のログイン画面。初回ログインでパスワード変更を求められたら新しいパスワードを設定する
export class LoginPage {
  constructor(private readonly page: Page) {}

  async login(email: string, password: string, newPassword?: string) {
    await this.page.goto("/");
    await this.page.getByRole("button", { name: "ログイン" }).click();
    await this.page.locator("#username").fill(email);
    await this.page.locator("#password").fill(password);
    await this.page.locator("#kc-login").click();

    if (newPassword) {
      await this.page.locator("#password-new").fill(newPassword);
      await this.page.locator("#password-confirm").fill(newPassword);
      await this.page.locator("input[type=submit], button[type=submit]").first().click();
    }
    await this.page.getByRole("heading", { name: "給与管理" }).waitFor();
  }
}
