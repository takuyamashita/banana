import type { ReactNode } from "react";

import { cx } from "./cx";

export function Card({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className={cx("bg-surface", "border", "radius-2", "p-y-5", "p-x-6", "m-y-5")} aria-label={title}>
      <h2 className={cx("m-t-0", "m-b-4", "fs-title")}>{title}</h2>
      {children}
    </section>
  );
}
