import { expect, type Page } from "@playwright/test";

export interface EntryValues {
  project: string;
  minutes: number;
}

/// 派遣社員が自分の勤怠を書いて申告する画面
export class TimesheetPage {
  constructor(private readonly page: Page) {}

  /// その月の勤務表を開く。派遣社員の登録は給与から出来事で届くので、届くまで開き直す
  async open(year: number, month: number) {
    await expect(async () => {
      await this.page.goto(`/timesheet?year=${year}&month=${month}`);
      await expect(this.page.getByRole("heading", { name: `${year}年${month}月の勤務表` })).toBeVisible({
        timeout: 2_000,
      });
    }).toPass({ timeout: 20_000 });
  }

  /// 1日目から順に1行ずつ書く
  async fill(entries: EntryValues[]) {
    for (const [i, entry] of entries.entries()) {
      await this.page.getByRole("button", { name: "行を足す" }).click();
      await this.page.getByLabel(`${i + 1}行目の案件`).selectOption({ label: entry.project });
      await this.page.getByLabel(`${i + 1}行目の稼働(分)`).fill(String(entry.minutes));
    }
  }

  async submit() {
    await this.page.getByRole("button", { name: "申告する" }).click();
  }
}
