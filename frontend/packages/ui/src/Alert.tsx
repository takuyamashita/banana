import type { ReactNode } from "react";

import { cx } from "./cx";

export function Alert({ tone = "error", children }: { tone?: "error" | "success"; children: ReactNode }) {
  return (
    <div
      role={tone === "error" ? "alert" : "status"}
      className={cx(
        "p-y-3",
        "p-x-4",
        "radius-1",
        "m-y-3",
        "border-current",
        tone === "error" ? "fg-error" : "fg-success",
      )}
    >
      {children}
    </div>
  );
}
