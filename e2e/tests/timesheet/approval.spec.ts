import { LoginPage } from "../../pages/login-page";
import { TimesheetApprovalPage } from "../../pages/timesheet-approval-page";
import { TimesheetPage } from "../../pages/timesheet-page";
import { ADMIN, adminApi, seedProject, seedStaff } from "../../support/api";
import { expect, test } from "../../support/fixtures";

// 承認した稼働が給与に届き、給与明細になるところまでは tests/journeys で通す
test("派遣社員が申告した勤怠を、管理者が承認できる", async ({ newPage }) => {
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

  // 派遣社員: 開き直すと承認済みになっている
  await staffPage.reload();
  await expect(staffPage.getByTestId("timesheet-status")).toHaveText("承認済み");
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
