import { useId, type InputHTMLAttributes, type ReactNode, type SelectHTMLAttributes } from "react";

import { cx } from "./cx";

/// 見出しと入力欄を縦に並べる枠
const fieldClass = () => cx("grid", "gap-1", "m-y-3");

export function Field({
  label,
  ...props
}: Omit<InputHTMLAttributes<HTMLInputElement>, "className" | "style"> & { label: string }) {
  const id = useId();
  return (
    <div className={fieldClass()}>
      <label htmlFor={id}>{label}</label>
      <input id={id} {...props} />
    </div>
  );
}

export function SelectField({
  label,
  children,
  ...props
}: Omit<SelectHTMLAttributes<HTMLSelectElement>, "className" | "style"> & { label: string; children: ReactNode }) {
  const id = useId();
  return (
    <div className={fieldClass()}>
      <label htmlFor={id}>{label}</label>
      <select id={id} {...props}>
        {children}
      </select>
    </div>
  );
}
