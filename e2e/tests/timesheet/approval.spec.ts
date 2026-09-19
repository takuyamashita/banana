import { LoginPage } from "../../pages/login-page";
import { PayslipPage } from "../../pages/payslip-page";
import { TimesheetApprovalPage } from "../../pages/timesheet-approval-page";
import { TimesheetPage } from "../../pages/timesheet-page";
import { ADMIN, adminApi, seedProject, seedStaff } from "../../support/api";
import { expect, test } from "../../support/fixtures";

test("派遣社員が申告した勤怠を管理者が承認すると、その稼働で給与明細を作れる", async ({ newPage }) => {
  const api = await adminApi();
  const staff = await seedStaff(api);
  const project = await seedProject(api);

  // 派遣社員: 初回ログインでパスワードを変え、9月の勤怠を書いて申告する
  const staffPage = await newPage();
  await new LoginPage(staffPage).login(staff.email, staff.temporaryPassword, "New-pass-12345");
  const timesheet = new TimesheetPage(staffPage);
  await timesheet.open(2026, 9);
  await timesheet.fill([
    { project: project.name, minutes: 480 },
    { project: project.name, minutes: 450 },
  ]);
  await expect(staffPage.getByTestId("timesheet-total")).toHaveText("15時間30分");
  await timesheet.submit();
  await expect(staffPage.getByText("申告しました。管理者の承認を待っています。")).toBeVisible();
  await expect(staffPage.getByTestId("timesheet-status")).toHaveText("申告済み(承認待ち)");

  // 管理者: 申告された勤務表を承認する
  const adminPage = await newPage();
  await new LoginPage(adminPage).login(ADMIN.email, ADMIN.password);
  const approval = new TimesheetApprovalPage(adminPage);
  await approval.open();
  await approval.approve(staff.displayName, 2026, 9);
  await expect(adminPage.getByText(`${staff.displayName}さんの2026年9月の勤務表を承認しました。`)).toBeVisible();

  // 承認した稼働は出来事で給与に届く(届くまで待つ)
  await expect
    .poll(
      async () =>
        (await api.payroll.getApprovedWork({ staffId: staff.staffId, payYear: 2026, payMonth: 9 })).work.length,
    )
    .toBe(1);

  // 管理者: 承認済みの勤怠から明細行を入れ、時給を入れて作る
  const payslips = new PayslipPage(adminPage);
  await payslips.open();
  await adminPage.getByLabel("派遣社員").selectOption({ label: `${staff.displayName}(${staff.email})` });
  await adminPage.getByLabel("年", { exact: true }).fill("2026");
  await adminPage.getByLabel("月", { exact: true }).fill("9");
  await expect(adminPage.getByText(`この月の承認済みの勤怠: ${project.name} 15時間30分`)).toBeVisible();
  await adminPage.getByRole("button", { name: "承認済みの勤怠から明細を入れる" }).click();
  const line = adminPage.getByRole("group", { name: "明細 1" });
  await expect(line.getByLabel("稼働(分)")).toHaveValue("930");
  await line.getByLabel("時給(円)").fill("1200");
  await adminPage.getByRole("button", { name: "作成する" }).click();
  // 930分 × 1200円 / 60 = 18,600円
  await expect(adminPage.getByTestId("payslip-total")).toHaveText("￥18,600");
});

test("差し戻された勤怠は、理由を見て直し、申告し直せる", async ({ newPage }) => {
  const api = await adminApi();
  const staff = await seedStaff(api);
  const project = await seedProject(api);

  const staffPage = await newPage();
  await new LoginPage(staffPage).login(staff.email, staff.temporaryPassword, "New-pass-12345");
  const timesheet = new TimesheetPage(staffPage);
  await timesheet.open(2026, 9);
  await timesheet.fill([{ project: project.name, minutes: 480 }]);
  await timesheet.submit();
  await expect(staffPage.getByTestId("timesheet-status")).toHaveText("申告済み(承認待ち)");

  const adminPage = await newPage();
  await new LoginPage(adminPage).login(ADMIN.email, ADMIN.password);
  const approval = new TimesheetApprovalPage(adminPage);
  await approval.open();
  await approval.sendBack(staff.displayName, 2026, 9, "9/1 は 450 分のはずです");
  await expect(adminPage.getByText(`${staff.displayName}さんの2026年9月の勤務表を差し戻しました。`)).toBeVisible();

  // 派遣社員: 開き直すと理由が見え、直して申告し直せる
  await staffPage.reload();
  await expect(staffPage.getByRole("alert")).toHaveText("差し戻されました: 9/1 は 450 分のはずです");
  await staffPage.getByLabel("1行目の稼働(分)").fill("450");
  await timesheet.submit();
  await expect(staffPage.getByTestId("timesheet-status")).toHaveText("申告済み(承認待ち)");
  await expect(staffPage.getByTestId("timesheet-total")).toHaveText("7時間30分");
});
