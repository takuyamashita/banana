import { buttonClass } from "@platform/ui";
import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";

import { AppLayout, type Menu } from "./AppLayout";

function renderAt(path: string, menu: Menu = "admin") {
  const root = createRootRoute({
    component: () => (
      <AppLayout email="admin@example.com" menu={menu} onSignOut={() => {}} signOutError={null}>
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

test("派遣社員には、自分の給与明細と勤怠のメニューを出す", async () => {
  renderAt("/timesheet", "staff");

  const nav = await screen.findByRole("navigation", { name: "メニュー" });
  expect(nav).toHaveTextContent("自分の給与明細勤怠");
  expect(screen.getByRole("link", { name: "勤怠" })).toHaveAttribute("aria-current", "page");
  expect(screen.queryByRole("link", { name: "派遣社員" })).toBeNull();
});
