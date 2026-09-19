// 動作確認の動画: 勤怠(timesheet)を別のサービスとして足した PR。
// 給与で登録した派遣社員が勤怠を申告し、管理者が承認すると、その稼働が給与に届いて給与明細の明細行に入る
import { LoginPage } from "../../../pages/login-page";
import { PayslipPage } from "../../../pages/payslip-page";
import { TimesheetApprovalPage } from "../../../pages/timesheet-approval-page";
import { TimesheetPage } from "../../../pages/timesheet-page";
import { ADMIN, adminApi, seedProject, seedStaff } from "../../../support/api";
import { caption, expect, test } from "../../../support/video";

test("申告した勤怠を管理者が承認し、その稼働で給与明細を作る", async ({ record }) => {
  const api = await adminApi();
  const staff = await seedStaff(api);
  const project = await seedProject(api);

  const self = await record("本人");
  await new LoginPage(self).login(staff.email, staff.temporaryPassword, "New-pass-12345");
  const timesheet = new TimesheetPage(self);
  await timesheet.open(2026, 9);
  await caption(
    self,
    "① 派遣社員の勤怠の画面(勤怠は別のサービス。派遣社員と案件の登録は、給与から出来事で届いている)",
    3500,
  );
  await timesheet.fill([
    { project: project.name, minutes: 480 },
    { project: project.name, minutes: 450 },
    { project: project.name, minutes: 480 },
  ]);
  await caption(self, "② 日ごと・案件ごとに稼働を書く(15分単位。合計は入力に合わせて変わる)", 3000);
  await timesheet.submit();
  await expect(self.getByTestId("timesheet-status")).toHaveText("申告済み(承認待ち)");
  await caption(self, "③ 申告すると書き直せなくなり、管理者の承認を待つ", 3000);

  const admin = await record("管理者");
  await new LoginPage(admin).login(ADMIN.email, ADMIN.password);
  const approval = new TimesheetApprovalPage(admin);
  await approval.open();
  await caption(admin, "④ 管理者の「勤怠の承認」に、申告された勤務表が並ぶ", 3000);
  await approval.approve(staff.displayName, 2026, 9);
  await expect(admin.getByText(`${staff.displayName}さんの2026年9月の勤務表を承認しました。`)).toBeVisible();
  await caption(admin, "⑤ 承認すると、案件ごとの稼働の合計が出来事で給与のサービスに届く", 3000);

  await expect
    .poll(
      async () =>
        (await api.payroll.getApprovedWork({ staffId: staff.staffId, payYear: 2026, payMonth: 9 })).work.length,
    )
    .toBe(1);
  await new PayslipPage(admin).open();
  await admin.getByLabel("派遣社員").selectOption({ label: `${staff.displayName}(${staff.email})` });
  await admin.getByLabel("年", { exact: true }).fill("2026");
  await admin.getByLabel("月", { exact: true }).fill("9");
  await expect(admin.getByText(/この月の承認済みの勤怠/)).toBeVisible();
  await caption(admin, "⑥ 給与明細の作成で、その月の承認済みの勤怠が見える", 3000);
  await admin.getByRole("button", { name: "承認済みの勤怠から明細を入れる" }).click();
  await caption(admin, "⑦ 明細行に案件と稼働が入る。時給だけを入れる", 2500);
  await admin.getByRole("group", { name: "明細 1" }).getByLabel("時給(円)").fill("1200");
  await admin.getByRole("button", { name: "作成する" }).click();
  await expect(admin.getByTestId("payslip-total")).toBeVisible();
  await admin.getByTestId("payslip-total").scrollIntoViewIfNeeded();
  await caption(admin, "⑧ 承認された稼働で給与明細ができる(1410分 × 1,200円 ÷ 60 = 28,200円)", 3500);
});

test("差し戻された勤怠は、理由を見て直し、申告し直す", async ({ record }) => {
  const api = await adminApi();
  const staff = await seedStaff(api);
  const project = await seedProject(api);

  const self = await record("本人");
  await new LoginPage(self).login(staff.email, staff.temporaryPassword, "New-pass-12345");
  const timesheet = new TimesheetPage(self);
  await timesheet.open(2026, 9);
  await timesheet.fill([{ project: project.name, minutes: 480 }]);
  await timesheet.submit();
  await expect(self.getByTestId("timesheet-status")).toHaveText("申告済み(承認待ち)");
  await caption(self, "① 派遣社員が勤怠を申告する", 2000);

  const admin = await record("管理者");
  await new LoginPage(admin).login(ADMIN.email, ADMIN.password);
  const approval = new TimesheetApprovalPage(admin);
  await approval.open();
  await approval.sheet(staff.displayName, 2026, 9).getByLabel("差し戻すときの理由").fill("9/1 は 450 分のはずです");
  await caption(admin, "② 管理者は、理由を付けて差し戻せる", 2500);
  await approval.sheet(staff.displayName, 2026, 9).getByRole("button", { name: "差し戻す" }).click();
  await expect(admin.getByText(/差し戻しました/)).toBeVisible();

  await self.reload();
  await expect(self.getByRole("alert")).toHaveText("差し戻されました: 9/1 は 450 分のはずです");
  await caption(self, "③ 派遣社員に理由が見え、勤務表は作成中に戻る", 3000);
  await self.getByLabel("1行目の稼働(分)").fill("450");
  await timesheet.submit();
  await expect(self.getByTestId("timesheet-status")).toHaveText("申告済み(承認待ち)");
  await caption(self, "④ 直して申告し直す", 2500);
});
