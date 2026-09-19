// 動作確認の動画の台本。場面ごとに字幕を出してから操作する。
// 確かめるテスト(tests/)と同じく、画面が変わって操作が通らなくなれば録画も失敗する
import { LoginPage } from "../pages/login-page";
import { PayslipPage } from "../pages/payslip-page";
import { ADMIN, adminApi, seedProject, seedStaff } from "../support/api";
import { caption, expect, test } from "../support/video";

test("管理者: ログイン後の戻り先・給与明細の作成と確定・画面の移動・セッション切れ", async ({ record }) => {
  const api = await adminApi();
  const staff = await seedStaff(api);
  const project = await seedProject(api);
  const staffLabel = `${staff.displayName}(${staff.email})`;
  const page = await record();

  await page.goto(`/payslips?staffId=${staff.staffId}`);
  await caption(page, "① ログイン前に、派遣社員を選んだ給与明細の URL を開く → ログイン画面へ");
  await page.getByRole("button", { name: "ログイン" }).click();
  await page.locator("#username").fill(ADMIN.email);
  await page.locator("#password").fill(ADMIN.password);
  await page.locator("#kc-login").click();
  await expect(page).toHaveURL(new RegExp(`/payslips\\?staffId=${staff.staffId}$`));
  await caption(page, "② ログイン後、開こうとしていた画面に戻る(派遣社員も選ばれたまま)", 3000);

  await caption(page, "③ 給与明細を作る(明細 2行)", 1200);
  const payslips = new PayslipPage(page);
  await payslips.create(staffLabel, 2026, 9, [
    { project: project.name, minutes: 9600, hourlyRate: 1501 },
    { project: project.name, minutes: 90, hourlyRate: 1500 },
  ]);
  await expect(page.getByTestId("payslip-total")).toHaveText("￥242,410");
  await page.getByTestId("payslip-total").scrollIntoViewIfNeeded();
  await caption(page, "④ 作成中として一覧に出る。合計 ￥242,410 を確かめる", 3000);

  await payslips.finalize(2026, 9);
  await expect(page.getByText(/確定済み/)).toBeVisible();
  await caption(page, "⑤ 確定する → 確定済みになり、確定ボタンが消える", 3000);

  await payslips.create(staffLabel, 2026, 9, [{ project: project.name, minutes: 60, hourlyRate: 1000 }]);
  await expect(page.getByRole("alert")).toHaveText("この月の給与明細は既にあります");
  await caption(page, "⑥ 同じ月はもう作れない(サーバーの文言をそのまま出す)", 3000);

  await page.evaluate(() => window.scrollTo(0, 0));
  await page.getByRole("link", { name: "派遣社員" }).click();
  await expect(page).toHaveURL(/\/staff$/);
  await caption(page, "⑦ メニューで画面を移る(URL が /staff に変わり、選んでいるメニューが強調される)");
  await page.getByRole("link", { name: "案件" }).click();
  await expect(page).toHaveURL(/\/projects$/);
  await caption(page, "⑧ 案件の画面(/projects)", 2000);
  await page.goBack();
  await page.goBack();
  await expect(page).toHaveURL(new RegExp(`/payslips\\?staffId=${staff.staffId}$`));
  await caption(page, "⑨ ブラウザの「戻る」で、派遣社員を選んだ給与明細の画面まで戻る", 3000);

  // トークンを失った状態を作り、まだ取っていないデータを取りに行かせる
  await page.evaluate(() => window.sessionStorage.clear());
  await page.getByLabel("派遣社員").selectOption({ index: 1 });
  await expect(page.getByRole("alert")).toHaveText("ログインの有効期限が切れました。もう一度ログインしてください。");
  await caption(page, "⑩ ログインが切れたら、知らせを出してログイン画面へ(戻り先は今の画面)", 3000);
});

test("派遣社員本人: 初回ログインで確定済みの自分の給与明細を見る", async ({ record }) => {
  const api = await adminApi();
  const staff = await seedStaff(api);
  const project = await seedProject(api);
  const { payslipId } = await api.payroll.createPayslip({
    staffId: staff.staffId,
    payYear: 2026,
    payMonth: 9,
    lines: [
      { projectId: project.projectId, workMinutes: 9600, hourlyRate: 1501n },
      { projectId: project.projectId, workMinutes: 90, hourlyRate: 1500n },
    ],
  });
  await api.payroll.finalizePayslip({ payslipId });
  const page = await record();

  await page.goto("/");
  await caption(page, "① 派遣社員本人がログインする(初回はパスワードを変える)", 2000);
  await new LoginPage(page).signIn(staff.email, staff.temporaryPassword, "New-pass-12345");
  await expect(page.getByTestId("payslip-total")).toHaveText("￥242,410");
  await caption(page, "② 確定した自分の給与明細だけが見える(案件名・稼働時間・金額)", 3500);
  await page.emulateMedia({ colorScheme: "dark" });
  await caption(page, "③ ダークモード(OS の設定に合わせる)");
});
