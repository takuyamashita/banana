import { expect, test } from "@playwright/test";

import { FinalizePage } from "../../pages/finalize-page";
import { LoginPage } from "../../pages/login-page";
import { ADMIN, adminApi, seedProject, seedStaff } from "../../support/api";

test("管理者が確定した給与明細を、本人がログインして見られる", async ({ browser }) => {
  const api = await adminApi();
  const staff = await seedStaff(api);
  const project = await seedProject(api);

  // 管理者: 給与を確定する
  const adminPage = await (await browser.newContext()).newPage();
  await new LoginPage(adminPage).login(ADMIN.email, ADMIN.password);
  const finalize = new FinalizePage(adminPage);
  await finalize.open();
  await finalize.finalize(`${staff.displayName}(${staff.email})`, 2026, 9, [
    { project: project.name, minutes: 9600, hourlyRate: 1501 },
    { project: project.name, minutes: 100, hourlyRate: 1500 },
  ]);
  await expect(adminPage.getByText(/給与明細 #\d+ を確定しました。/)).toBeVisible();
  // 9600分×1501円/60 = 240,160円、100分→90分×1500円/60 = 2,250円
  await expect(adminPage.getByTestId("payslip-total")).toHaveText("￥242,410");

  // 同じ月をもう一度確定するとエラーになる
  await finalize.finalize(`${staff.displayName}(${staff.email})`, 2026, 9, [
    { project: project.name, minutes: 60, hourlyRate: 1000 },
  ]);
  await expect(adminPage.getByRole("alert")).toHaveText("この月の給与明細は既に確定しています");

  // 本人: 初回ログインでパスワードを変え、自分の明細を見る
  const staffPage = await (await browser.newContext()).newPage();
  await new LoginPage(staffPage).login(staff.email, staff.temporaryPassword, "New-pass-12345");
  await expect(staffPage.getByRole("heading", { name: "自分の給与明細" })).toBeVisible();
  await expect(staffPage.getByTestId("payslip-total")).toHaveText("￥242,410");
});

test("他の派遣社員の給与明細は見えない", async ({ browser }) => {
  const api = await adminApi();
  const owner = await seedStaff(api);
  const other = await seedStaff(api);
  const project = await seedProject(api);
  await api.payroll.finalizePayslip({
    staffId: owner.staffId,
    payYear: 2026,
    payMonth: 9,
    lines: [{ projectId: project.projectId, workMinutes: 600, hourlyRate: 1200n }],
  });

  const page = await (await browser.newContext()).newPage();
  await new LoginPage(page).login(other.email, other.temporaryPassword, "New-pass-12345");
  await expect(page.getByText("まだ確定した給与明細はありません。")).toBeVisible();
  await expect(page.getByTestId("payslip-total")).toHaveCount(0);
});
