import type { Page } from "@playwright/test";

/// Keycloak のログイン画面。初回ログインでパスワード変更を求められたら新しいパスワードを設定する
export class LoginPage {
  constructor(private readonly page: Page) {}

  async login(email: string, password: string, newPassword?: string) {
    await this.page.goto("/");
    await this.signIn(email, password, newPassword);
  }

  /// ログイン画面が出ている状態から、IdP でログインしてアプリに戻る
  async signIn(email: string, password: string, newPassword?: string) {
    await this.page.getByRole("button", { name: "ログイン" }).click();
    await this.page.locator("#username").fill(email);
    await this.page.locator("#password").fill(password);
    await this.page.locator("#kc-login").click();

    if (newPassword) {
      await this.page.locator("#password-new").fill(newPassword);
      await this.page.locator("#password-confirm").fill(newPassword);
      await this.page.locator("input[type=submit], button[type=submit]").first().click();
    }
    // ログインした人の画面の枠が出るまで待つ(見出しは開いた画面のシステムで変わる)
    await this.page.getByRole("button", { name: "ログアウト" }).waitFor();
  }
}
