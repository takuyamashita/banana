import { Alert, Button, buttonClass, cx, pageClass } from "@platform/ui";
import { Link } from "@tanstack/react-router";
import { type ReactNode, useId } from "react";

/// 画面がどのシステム(サービス)のものか。システムごとに名前と色を変え、今どちらを触っているかを分かるようにする
export type System = "payroll" | "timesheet";

const systems = {
  payroll: { title: "給与管理", label: "給与" },
  timesheet: { title: "勤怠管理", label: "勤怠" },
} as const satisfies Record<System, { title: string; label: string }>;

/// メニュー。URL ごとに画面が分かれ、システムごとにまとめて並べる
const menus = {
  admin: {
    groups: [
      {
        system: "payroll",
        items: [
          { to: "/payslips", label: "給与明細" },
          { to: "/staff", label: "派遣社員" },
          { to: "/projects", label: "案件" },
        ],
      },
      { system: "timesheet", items: [{ to: "/timesheets", label: "勤怠の承認" }] },
    ],
  },
  staff: {
    groups: [
      { system: "payroll", items: [{ to: "/me", label: "自分の給与明細" }] },
      { system: "timesheet", items: [{ to: "/timesheet", label: "勤怠" }] },
    ],
  },
} as const;

/// どのメニューを出すか。派遣社員として登録されていない利用者には出さない
export type Menu = keyof typeof menus | "none";

type MenuItem = (typeof menus)[keyof typeof menus]["groups"][number]["items"][number];

/// ログインした人の画面の枠。開いている画面のシステムの名前と色で囲み、管理者と派遣社員にはそれぞれのメニューを出す
export function AppLayout({
  system,
  email,
  menu,
  onSignOut,
  signOutError,
  children,
}: {
  system: System;
  email: string;
  menu: Menu;
  onSignOut: () => void;
  signOutError: string | null;
  children: ReactNode;
}) {
  return (
    <main className={pageClass()} data-system={system}>
      <header
        className={cx(
          "flex",
          "items-center",
          "gap-4",
          "wrap",
          "p-y-3",
          "p-x-5",
          "radius-2",
          "border-accent",
          "bg-accent-tint",
        )}
      >
        <h1 className={cx("m-r-auto", "fg-accent")}>{systems[system].title}</h1>
        <span className={cx("fg-muted")}>{email}</span>
        <Button variant="secondary" onClick={onSignOut}>
          ログアウト
        </Button>
      </header>
      {signOutError && <Alert>{signOutError}</Alert>}
      {menu !== "none" && (
        <div className={cx("flex", "wrap", "gap-4", "items-center", "m-y-3")}>
          {menus[menu].groups.map((group) => (
            <SystemMenu key={group.system} system={group.system} items={group.items} />
          ))}
        </div>
      )}
      {children}
    </main>
  );
}

/// 1つのシステムのメニュー。どのシステムの画面かが分かるよう、システムの名前を添えてその色で並べる
function SystemMenu({ system, items }: { system: System; items: readonly MenuItem[] }) {
  const labelId = useId();
  return (
    <nav aria-labelledby={labelId} data-system={system} className={cx("flex", "wrap", "gap-3", "items-center")}>
      <span id={labelId} className={cx("fg-accent")}>
        {systems[system].label}
      </span>
      {items.map(({ to, label }) => (
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
  );
}
