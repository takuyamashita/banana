import { PayslipPage } from "../../pages/payslip-page";
import { LoginPage } from "../../pages/login-page";
import { ADMIN, adminApi, seedProject, seedStaff } from "../../support/api";
import { expect, test } from "../../support/fixtures";

test("管理者が作成して確定した給与明細を、本人がログインして見られる", async ({ newPage }) => {
  const api = await adminApi();
  const staff = await seedStaff(api);
  const project = await seedProject(api);

  // 管理者: 給与明細を作成し、内容を確かめてから確定する
  const adminPage = await newPage();
  await new LoginPage(adminPage).login(ADMIN.email, ADMIN.password);
  const payslips = new PayslipPage(adminPage);
  await payslips.open();
  await payslips.create(`${staff.displayName}(${staff.email})`, 2026, 9, [
    { project: project.name, minutes: 9600, hourlyRate: 1501 },
    { project: project.name, minutes: 90, hourlyRate: 1500 },
  ]);
  await expect(adminPage.getByText(/給与明細 #\d+ を作成しました。/)).toBeVisible();
  await expect(adminPage.getByText(/作成中/)).toBeVisible();
  // 9600分×1501円/60 = 240,160円、90分×1500円/60 = 2,250円
  await expect(adminPage.getByTestId("payslip-total")).toHaveText("￥242,410");

  await payslips.finalize(2026, 9);
  await expect(adminPage.getByText(/給与明細 #\d+ を確定しました。/)).toBeVisible();
  await expect(adminPage.getByText(/確定済み/)).toBeVisible();
  // 確定済みにはもう確定のボタンが出ない
  await expect(adminPage.getByRole("button", { name: "2026年9月分を確定する" })).toHaveCount(0);

  // 同じ月の給与明細はもう作れない
  await payslips.create(`${staff.displayName}(${staff.email})`, 2026, 9, [
    { project: project.name, minutes: 60, hourlyRate: 1000 },
  ]);
  await expect(adminPage.getByRole("alert")).toHaveText("この月の給与明細は既にあります");

  // 本人: 初回ログインでパスワードを変え、自分の明細を見る
  const staffPage = await newPage();
  await new LoginPage(staffPage).login(staff.email, staff.temporaryPassword, "New-pass-12345");
  await expect(staffPage.getByRole("heading", { name: "自分の給与明細" })).toBeVisible();
  await expect(staffPage.getByTestId("payslip-total")).toHaveText("￥242,410");
  // 明細行には案件名と、時間で表した稼働が出る
  await expect(staffPage.getByRole("cell", { name: project.name })).toHaveCount(2);
  await expect(staffPage.getByRole("cell", { name: "160時間" })).toBeVisible();
});

// 画面は自分の給与明細しか取りに行かないので、ここで確かめるのは「他人の明細が自分の画面に出ない」こと。
// 他人の明細を API で取れないことは、API テスト・スモークテストで確かめている
test("他の派遣社員の給与明細は、自分の画面に出ない", async ({ newPage }) => {
  const api = await adminApi();
  const owner = await seedStaff(api);
  const other = await seedStaff(api);
  const project = await seedProject(api);
  const { payslipId } = await api.payroll.createPayslip({
    staffId: owner.staffId,
    payYear: 2026,
    payMonth: 9,
    lines: [{ projectId: project.projectId, workMinutes: 600, hourlyRate: 1200n }],
  });
  await api.payroll.finalizePayslip({ payslipId });

  const page = await newPage();
  await new LoginPage(page).login(other.email, other.temporaryPassword, "New-pass-12345");
  await expect(page.getByText("まだ確定した給与明細はありません。")).toBeVisible();
  await expect(page.getByTestId("payslip-total")).toHaveCount(0);
});

test("作成中の給与明細は本人に見えず、確定すると見える", async ({ newPage }) => {
  const api = await adminApi();
  const staff = await seedStaff(api);
  const project = await seedProject(api);
  const { payslipId } = await api.payroll.createPayslip({
    staffId: staff.staffId,
    payYear: 2026,
    payMonth: 9,
    lines: [{ projectId: project.projectId, workMinutes: 600, hourlyRate: 1200n }],
  });

  // 作成中の間は、本人の画面に出ない
  const page = await newPage();
  await new LoginPage(page).login(staff.email, staff.temporaryPassword, "New-pass-12345");
  await expect(page.getByText("まだ確定した給与明細はありません。")).toBeVisible();
  await expect(page.getByTestId("payslip-total")).toHaveCount(0);

  // 管理者が確定すると、本人の画面に出る(ログインはタブの sessionStorage に残っている)
  await api.payroll.finalizePayslip({ payslipId });
  await page.reload();
  await expect(page.getByRole("heading", { name: "自分の給与明細" })).toBeVisible();
  // 600分×1200円/60 = 12,000円
  await expect(page.getByTestId("payslip-total")).toHaveText("￥12,000");
  await expect(page.getByText(/確定済み/)).toBeVisible();
});
