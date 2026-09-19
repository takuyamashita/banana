import { buttonClass } from "@platform/ui";
import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, within } from "@testing-library/react";

import { AppLayout, type Menu, type System } from "./AppLayout";

function renderAt(path: string, menu: Menu = "admin", system: System = "payroll") {
  const root = createRootRoute({
    component: () => (
      <AppLayout system={system} email="admin@example.com" menu={menu} onSignOut={() => {}} signOutError={null}>
        <Outlet />
      </AppLayout>
    ),
  });
  const pages = ["/payslips", "/timesheets", "/staff", "/projects", "/me", "/timesheet"].map((page) =>
    createRoute({ getParentRoute: () => root, path: page, component: () => null }),
  );
  const router = createRouter({
    routeTree: root.addChildren(pages),
    history: createMemoryHistory({ initialEntries: [path] }),
  });
  render(<RouterProvider router={router} />);
}

test("開いている画面のメニューだけが強調される(選んでいないときの見た目と混ざらない)", async () => {
  renderAt("/staff");

  const current = await screen.findByRole("link", { name: "派遣社員" });
  expect(current).toHaveAttribute("aria-current", "page");
  expect(current).toHaveClass(buttonClass("primary"), { exact: true });
  const other = screen.getByRole("link", { name: "給与明細" });
  expect(other).not.toHaveAttribute("aria-current");
  expect(other).toHaveClass(buttonClass("secondary"), { exact: true });
});

test("派遣社員には、給与(自分の給与明細)と勤怠のメニューを出す", async () => {
  renderAt("/timesheet", "staff", "timesheet");

  expect(await screen.findByRole("navigation", { name: "給与" })).toHaveTextContent("給与自分の給与明細");
  const timesheet = screen.getByRole("navigation", { name: "勤怠" });
  expect(within(timesheet).getByRole("link", { name: "勤怠" })).toHaveAttribute("aria-current", "page");
  expect(screen.queryByRole("link", { name: "派遣社員" })).toBeNull();
});

test("開いている画面のシステムの名前を見出しにし、色をそのシステムのものにする", async () => {
  renderAt("/timesheets", "admin", "timesheet");

  expect(await screen.findByRole("heading", { name: "勤怠管理" })).toBeVisible();
  expect(screen.getByRole("main")).toHaveAttribute("data-system", "timesheet");
  // メニューは、どの画面を開いていても、それぞれのシステムの色で並ぶ
  expect(screen.getByRole("navigation", { name: "給与" })).toHaveAttribute("data-system", "payroll");
  expect(screen.getByRole("navigation", { name: "勤怠" })).toHaveAttribute("data-system", "timesheet");
});
