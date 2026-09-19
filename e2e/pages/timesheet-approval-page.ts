import type { Page } from "@playwright/test";

/// 管理者が、申告された勤務表を承認するか差し戻す画面
export class TimesheetApprovalPage {
  constructor(private readonly page: Page) {}

  async open() {
    await this.page.getByRole("link", { name: "勤怠の承認" }).click();
  }

  sheet(staffName: string, year: number, month: number) {
    return this.page.getByRole("region", { name: `${staffName}さんの${year}年${month}月の勤務表` });
  }

  async approve(staffName: string, year: number, month: number) {
    await this.sheet(staffName, year, month).getByRole("button", { name: "承認する" }).click();
  }

  async sendBack(staffName: string, year: number, month: number, reason: string) {
    const sheet = this.sheet(staffName, year, month);
    await sheet.getByLabel("差し戻すときの理由").fill(reason);
    await sheet.getByRole("button", { name: "差し戻す" }).click();
  }
}
