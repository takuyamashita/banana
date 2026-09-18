import type { Page } from "@playwright/test";

export interface LineValues {
  project: string;
  minutes: number;
  hourlyRate: number;
}

export class FinalizePage {
  constructor(private readonly page: Page) {}

  async open() {
    await this.page.getByRole("button", { name: "給与確定" }).click();
  }

  async finalize(staffLabel: string, year: number, month: number, lines: LineValues[]) {
    await this.page.getByLabel("派遣社員").selectOption({ label: staffLabel });
    await this.page.getByLabel("年", { exact: true }).fill(String(year));
    await this.page.getByLabel("月", { exact: true }).fill(String(month));
    for (const [i, line] of lines.entries()) {
      if (i > 0) await this.page.getByRole("button", { name: "明細を追加" }).click();
      const fieldset = this.page.getByRole("group", { name: `明細 ${i + 1}` });
      await fieldset.getByLabel("案件").selectOption({ label: line.project });
      await fieldset.getByLabel("稼働(分)").fill(String(line.minutes));
      await fieldset.getByLabel("時給(円)").fill(String(line.hourlyRate));
    }
    await this.page.getByRole("button", { name: "確定する" }).click();
  }
}
