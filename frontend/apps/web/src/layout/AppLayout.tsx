import { Alert, Button, buttonClass, clusterClass, cx, pageClass } from "@platform/ui";
import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";

/// メニュー。URL ごとに画面が分かれる
const menus = {
  admin: {
    label: "管理メニュー",
    items: [
      { to: "/payslips", label: "給与明細" },
      { to: "/timesheets", label: "勤怠の承認" },
      { to: "/staff", label: "派遣社員" },
      { to: "/projects", label: "案件" },
    ],
  },
  staff: {
    label: "メニュー",
    items: [
      { to: "/me", label: "自分の給与明細" },
      { to: "/timesheet", label: "勤怠" },
    ],
  },
} as const;

/// どのメニューを出すか。派遣社員として登録されていない利用者には出さない
export type Menu = keyof typeof menus | "none";

/// ログインした人の画面の枠。管理者と派遣社員には、それぞれのメニューを出す
export function AppLayout({
  email,
  menu,
  onSignOut,
  signOutError,
  children,
}: {
  email: string;
  menu: Menu;
  onSignOut: () => void;
  signOutError: string | null;
  children: ReactNode;
}) {
  return (
    <main className={pageClass()}>
      <header className={cx("flex", "items-center", "gap-4", "wrap")}>
        <h1 className={cx("m-r-auto")}>給与管理</h1>
        <span className={cx("fg-muted")}>{email}</span>
        <Button variant="secondary" onClick={onSignOut}>
          ログアウト
        </Button>
      </header>
      {signOutError && <Alert>{signOutError}</Alert>}
      {menu !== "none" && (
        <nav className={clusterClass()} aria-label={menus[menu].label}>
          {menus[menu].items.map(({ to, label }) => (
            <Link
              key={to}
              to={to}
              // 選んでいるときのクラスは className に足される(置き換わらない)ので、見た目は両方とも状態ごとに渡す
              inactiveProps={{ className: buttonClass("secondary") }}
              activeProps={{ className: buttonClass("primary"), "aria-current": "page" }}
            >
              {label}
            </Link>
          ))}
        </nav>
      )}
      {children}
    </main>
  );
}
