import type { ReactNode } from "react";

export function Card({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="ui-card" aria-label={title}>
      <h2 className="ui-card__title">{title}</h2>
      {children}
    </section>
  );
}
