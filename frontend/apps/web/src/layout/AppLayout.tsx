import { Alert, Button, buttonClass, clusterClass, cx, pageClass } from "@platform/ui";
import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";

/// 管理者のメニュー。URL ごとに画面が分かれる
const adminMenu = [
  { to: "/payslips", label: "給与明細" },
  { to: "/staff", label: "派遣社員" },
  { to: "/projects", label: "案件" },
] as const;

/// ログインした人の画面の枠。管理者にはメニューを出す
export function AppLayout({
  email,
  isAdmin,
  onSignOut,
  signOutError,
  children,
}: {
  email: string;
  isAdmin: boolean;
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
      {isAdmin && (
        <nav className={clusterClass()} aria-label="管理メニュー">
          {adminMenu.map(({ to, label }) => (
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
