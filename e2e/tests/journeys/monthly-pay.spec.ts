import { expect, type Page } from "@playwright/test";

import { LoginPage } from "../../pages/login-page";
import { PayslipPage } from "../../pages/payslip-page";
import { ProjectAdminPage } from "../../pages/project-admin-page";
import { StaffAdminPage } from "../../pages/staff-admin-page";
import { TimesheetApprovalPage } from "../../pages/timesheet-approval-page";
import { TimesheetPage } from "../../pages/timesheet-page";
import { ADMIN, unique } from "../../support/api";
import { test } from "../../support/fixtures";

/// 画面の枠が、どのシステム(給与・勤怠)の名前と色で出ているか
async function expectSystem(page: Page, title: "給与管理" | "勤怠管理", system: "payroll" | "timesheet") {
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(title);
  await expect(page.getByRole("main")).toHaveAttribute("data-system", system);
}

const headingColor = (page: Page) =>
  page.getByRole("heading", { level: 1 }).evaluate((el) => getComputedStyle(el).color);

// 2つのサービスを画面だけで通す。データは API で入れず、サービスの間は出来事で届くのを待つ
test("管理者が登録した派遣社員が勤怠を申告し、承認された稼働の給与明細を確定すると、本人が見られる", async ({
  newPage,
}) => {
  const projectName = unique("案件");
  const displayName = unique("派遣");
  const email = `${unique("staff")}@example.com`;

  // 管理者(給与): 案件と派遣社員を登録する。勤怠には出来事で届く
  const adminPage = await newPage();
  await new LoginPage(adminPage).login(ADMIN.email, ADMIN.password);
  await expectSystem(adminPage, "給与管理", "payroll");
  const payrollColor = await headingColor(adminPage);
  const projects = new ProjectAdminPage(adminPage);
  await projects.open();
  await projects.register(projectName);
  const staffAdmin = new StaffAdminPage(adminPage);
  await staffAdmin.open();
  const staffId = await staffAdmin.register(email, displayName, "Temp-pass-1");

  // 派遣社員(勤怠): 初回ログインでパスワードを変え、メニューから勤怠へ移って9月分を書いて申告する
  const staffPage = await newPage();
  await new LoginPage(staffPage).login(email, "Temp-pass-1", "New-pass-12345");
  await expectSystem(staffPage, "給与管理", "payroll");
  await staffPage.getByRole("link", { name: "勤怠", exact: true }).click();
  await expectSystem(staffPage, "勤怠管理", "timesheet");
  // 給与と勤怠は、見出しの色でも見分けられる
  expect(await headingColor(staffPage)).not.toBe(payrollColor);
  const timesheet = new TimesheetPage(staffPage);
  await timesheet.open(2026, 9);
  await timesheet.fill([
    { project: projectName, minutes: 480 },
    { project: projectName, minutes: 450 },
  ]);
  await timesheet.submit();
  await expect(staffPage.getByTestId("timesheet-status")).toHaveText("申告済み(承認待ち)");

  // 管理者(勤怠): 申告された勤務表を承認する。承認した稼働は給与に出来事で届く
  const approval = new TimesheetApprovalPage(adminPage);
  await approval.open();
  await expectSystem(adminPage, "勤怠管理", "timesheet");
  await approval.approve(displayName, 2026, 9);
  await expect(adminPage.getByText(`${displayName}さんの2026年9月の勤務表を承認しました。`)).toBeVisible();

  // 管理者(給与): 承認済みの勤怠から明細行を入れて作り、確定する。届くまでは開き直す
  const payslips = new PayslipPage(adminPage);
  await payslips.open();
  await expectSystem(adminPage, "給与管理", "payroll");
  await expect(async () => {
    await adminPage.goto(`/payslips?staffId=${staffId}`);
    await adminPage.getByLabel("年", { exact: true }).fill("2026");
    await adminPage.getByLabel("月", { exact: true }).fill("9");
    await expect(adminPage.getByText(`この月の承認済みの勤怠: ${projectName} 15時間30分`)).toBeVisible({
      timeout: 2_000,
    });
  }).toPass({ timeout: 20_000 });
  await adminPage.getByRole("button", { name: "承認済みの勤怠から明細を入れる" }).click();
  await adminPage.getByRole("group", { name: "明細 1" }).getByLabel("時給(円)").fill("1200");
  await adminPage.getByRole("button", { name: "作成する" }).click();
  // 930分 × 1200円 / 60 = 18,600円
  await expect(adminPage.getByTestId("payslip-total")).toHaveText("￥18,600");
  await payslips.finalize(2026, 9);
  await expect(adminPage.getByText(/給与明細 #\d+ を確定しました。/)).toBeVisible();

  // 派遣社員(給与): 自分の給与明細で、申告した稼働の明細を見る
  await staffPage.getByRole("link", { name: "自分の給与明細" }).click();
  await expectSystem(staffPage, "給与管理", "payroll");
  await expect(staffPage.getByTestId("payslip-total")).toHaveText("￥18,600");
  await expect(staffPage.getByRole("cell", { name: projectName })).toBeVisible();
  await expect(staffPage.getByRole("cell", { name: "15時間30分" })).toBeVisible();
});
