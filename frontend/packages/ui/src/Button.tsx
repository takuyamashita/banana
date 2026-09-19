import type { ButtonHTMLAttributes } from "react";

import { cx } from "./cx";

export type ButtonVariant = "primary" | "secondary";

/// ボタンの見た目。ボタンの形をしたリンク(画面の切り替え)にも使う
export function buttonClass(variant: ButtonVariant): string {
  return cx(
    "p-y-2",
    "p-x-5",
    "radius-1",
    "border-accent",
    variant === "primary" ? "bg-accent" : "bg-transparent",
    variant === "primary" ? "fg-on-accent" : "fg-accent",
  );
}

export function Button({
  variant = "primary",
  ...props
}: Omit<ButtonHTMLAttributes<HTMLButtonElement>, "className" | "style"> & { variant?: ButtonVariant }) {
  return <button type="button" className={buttonClass(variant)} {...props} />;
}
