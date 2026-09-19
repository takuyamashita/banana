// 動作確認の動画: 給与と勤怠の2つのシステムを、画面だけで通す。
// 管理者が給与で登録した派遣社員が勤怠で申告し、管理者が承認した稼働で給与明細を作って確定すると、本人が見られる。
// 画面の枠の名前と色(給与は青、勤怠は橙)で、今どちらのシステムを触っているかが分かる
import { LoginPage } from "../../../pages/login-page";
import { PayslipPage } from "../../../pages/payslip-page";
import { ProjectAdminPage } from "../../../pages/project-admin-page";
import { StaffAdminPage } from "../../../pages/staff-admin-page";
import { TimesheetApprovalPage } from "../../../pages/timesheet-approval-page";
import { TimesheetPage } from "../../../pages/timesheet-page";
import { ADMIN, unique } from "../../../support/api";
import { caption, expect, test } from "../../../support/video";

test("給与で登録した派遣社員が勤怠を申告し、承認された稼働の給与明細を本人が見る", async ({ record }) => {
  const projectName = unique("案件");
  const displayName = unique("派遣");
  const email = `${unique("staff")}@example.com`;

  // 管理者(給与): 案件と派遣社員を登録する
  const admin = await record("管理者");
  await new LoginPage(admin).login(ADMIN.email, ADMIN.password);
  await caption(admin, "① 管理者がログインする。給与のシステムは青い枠(メニューもシステムごとに色が分かれる)", 3500);
  const projects = new ProjectAdminPage(admin);
  await projects.open();
  await projects.register(projectName);
  await caption(admin, "② 給与: 案件を登録する", 2500);
  const staffAdmin = new StaffAdminPage(admin);
  await staffAdmin.open();
  const staffId = await staffAdmin.register(email, displayName, "Temp-pass-1");
  await caption(admin, "③ 給与: 派遣社員を登録する。案件と派遣社員は、出来事で勤怠のシステムに届く", 3500);

  // 派遣社員(勤怠): 初回ログインでパスワードを変え、勤怠を書いて申告する
  const self = await record("本人");
  await new LoginPage(self).login(email, "Temp-pass-1", "New-pass-12345");
  await caption(self, "④ 派遣社員が初回ログインする。最初は給与の「自分の給与明細」(まだ何もない)", 3000);
  await self.getByRole("link", { name: "勤怠", exact: true }).click();
  const timesheet = new TimesheetPage(self);
  await timesheet.open(2026, 9);
  await expect(self.getByRole("heading", { level: 1 })).toHaveText("勤怠管理");
  await caption(self, "⑤ メニューの「勤怠」へ。勤怠のシステムは橙の枠になる", 3000);
  await timesheet.fill([
    { project: projectName, minutes: 480 },
    { project: projectName, minutes: 450 },
  ]);
  await caption(self, "⑥ 勤怠: 給与で登録された案件を選んで、日ごとの稼働を書く", 2500);
  await timesheet.submit();
  await expect(self.getByTestId("timesheet-status")).toHaveText("申告済み(承認待ち)");
  await caption(self, "⑦ 勤怠: 申告する。管理者の承認を待つ", 2500);

  // 管理者(勤怠): 承認する
  const approval = new TimesheetApprovalPage(admin);
  await approval.open();
  await caption(admin, "⑧ 管理者はメニューの「勤怠の承認」へ。ここは勤怠のシステムなので橙の枠", 3000);
  await approval.approve(displayName, 2026, 9);
  await expect(admin.getByText(`${displayName}さんの2026年9月の勤務表を承認しました。`)).toBeVisible();
  await caption(admin, "⑨ 勤怠: 承認する。案件ごとの稼働の合計が、出来事で給与のシステムに届く", 3500);

  // 管理者(給与): 承認済みの勤怠から給与明細を作って確定する(届くまでは開き直す)
  const payslips = new PayslipPage(admin);
  await payslips.open();
  await expect(async () => {
    await admin.goto(`/payslips?staffId=${staffId}`);
    await admin.getByLabel("年", { exact: true }).fill("2026");
    await admin.getByLabel("月", { exact: true }).fill("9");
    await expect(admin.getByText(`この月の承認済みの勤怠: ${projectName} 15時間30分`)).toBeVisible({
      timeout: 2_000,
    });
  }).toPass({ timeout: 20_000 });
  await caption(admin, "⑩ 給与(青)に戻ると、その月の承認済みの勤怠が見える", 3000);
  await admin.getByRole("button", { name: "承認済みの勤怠から明細を入れる" }).click();
  await admin.getByRole("group", { name: "明細 1" }).getByLabel("時給(円)").fill("1200");
  await caption(admin, "⑪ 給与: 承認された稼働を明細行に入れ、時給だけを入れる", 2500);
  await admin.getByRole("button", { name: "作成する" }).click();
  await expect(admin.getByTestId("payslip-total")).toHaveText("￥18,600");
  await payslips.finalize(2026, 9);
  await expect(admin.getByText(/給与明細 #\d+ を確定しました。/)).toBeVisible();
  await caption(admin, "⑫ 給与: 作成して確定する(930分 × 1,200円 ÷ 60 = 18,600円)", 3000);

  // 派遣社員(給与): 自分の給与明細を見る
  await self.getByRole("link", { name: "自分の給与明細" }).click();
  await expect(self.getByTestId("payslip-total")).toHaveText("￥18,600");
  await expect(self.getByRole("cell", { name: "15時間30分" })).toBeVisible();
  await caption(self, "⑬ 本人がメニューの「自分の給与明細」(青)へ。申告した稼働で確定した明細が見える", 3500);
});
