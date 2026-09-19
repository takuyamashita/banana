import { LoginPage } from "../../pages/login-page";
import { ADMIN, adminApi, seedStaff } from "../../support/api";
import { expect, test } from "../../support/fixtures";

test("ログイン前に開いた URL に、ログイン後に戻る(選んだ派遣社員も URL から戻る)", async ({ page }) => {
  const staff = await seedStaff(await adminApi());

  await page.goto(`/payslips?staffId=${staff.staffId}`);
  await expect(page).toHaveURL(/\/login\?returnTo=/);
  await new LoginPage(page).signIn(ADMIN.email, ADMIN.password);

  await expect(page).toHaveURL(new RegExp(`/payslips\\?staffId=${staff.staffId}$`));
  await expect(page.getByLabel("派遣社員")).toHaveValue(String(staff.staffId));
});

test("メニューで移った画面から、ブラウザの「戻る」で前の画面に戻る", async ({ page }) => {
  await new LoginPage(page).login(ADMIN.email, ADMIN.password);
  await expect(page).toHaveURL(/\/payslips$/);

  await page.getByRole("link", { name: "案件" }).click();
  await expect(page).toHaveURL(/\/projects$/);
  await expect(page.getByRole("link", { name: "案件" })).toHaveAttribute("aria-current", "page");

  await page.goBack();
  await expect(page).toHaveURL(/\/payslips$/);
  await expect(page.getByRole("link", { name: "給与明細" })).toHaveAttribute("aria-current", "page");
});
