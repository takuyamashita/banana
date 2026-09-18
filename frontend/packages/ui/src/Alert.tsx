import type { ReactNode } from "react";

export function Alert({ tone = "error", children }: { tone?: "error" | "success"; children: ReactNode }) {
  return (
    <div role={tone === "error" ? "alert" : "status"} className={`ui-alert ui-alert--${tone}`}>
      {children}
    </div>
  );
}
