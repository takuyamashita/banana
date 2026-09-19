import { expect, type Page } from "@playwright/test";

/// 管理者が派遣社員を登録する画面(給与)
export class StaffAdminPage {
  constructor(private readonly page: Page) {}

  async open() {
    await this.page.getByRole("link", { name: "派遣社員" }).click();
  }

  /// 登録し、登録した派遣社員の番号を返す
  async register(email: string, displayName: string, temporaryPassword: string): Promise<string> {
    const form = this.page.getByRole("form", { name: "派遣社員の登録" });
    await form.getByLabel("メールアドレス").fill(email);
    await form.getByLabel("表示名").fill(displayName);
    await form.getByLabel("仮パスワード").fill(temporaryPassword);
    await form.getByRole("button", { name: "登録する" }).click();
    const created = this.page.getByText(/派遣社員 #\d+ を登録しました。/);
    await expect(created).toBeVisible();
    return /#(\d+)/.exec((await created.textContent()) ?? "")?.[1] ?? "";
  }
}
