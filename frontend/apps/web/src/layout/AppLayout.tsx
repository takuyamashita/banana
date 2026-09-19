import { Alert, Button } from "@platform/ui";
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
    <main className="layout">
      <header className="header">
        <h1>給与管理</h1>
        <span className="who">{email}</span>
        <Button variant="secondary" onClick={onSignOut}>
          ログアウト
        </Button>
      </header>
      {signOutError && <Alert>{signOutError}</Alert>}
      {isAdmin && (
        <nav className="tabs" aria-label="管理メニュー">
          {adminMenu.map(({ to, label }) => (
            <Link
              key={to}
              to={to}
              className="ui-button ui-button--secondary"
              activeProps={{ className: "ui-button ui-button--primary", "aria-current": "page" }}
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
