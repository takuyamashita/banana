import { expect, type Page } from "@playwright/test";

/// 管理者が案件を登録する画面(給与)
export class ProjectAdminPage {
  constructor(private readonly page: Page) {}

  async open() {
    await this.page.getByRole("link", { name: "案件" }).click();
  }

  async register(name: string) {
    const form = this.page.getByRole("form", { name: "案件の登録" });
    await form.getByLabel("案件名").fill(name);
    await form.getByRole("button", { name: "登録する" }).click();
    await expect(this.page.getByRole("list", { name: "登録済みの案件" })).toContainText(name);
  }
}
