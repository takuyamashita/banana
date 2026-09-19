import { Alert } from "@platform/ui";
import type { ErrorComponentProps } from "@tanstack/react-router";

import { errorMessage } from "../lib/errors";

/// 画面のデータを取っている間
export function RoutePending() {
  return <output>読み込み中…</output>;
}

/// 画面を開けなかった(データを取れなかったなど)
export function RouteError({ error }: ErrorComponentProps) {
  return <Alert>{errorMessage(error)}</Alert>;
}

export function NotFound() {
  return <Alert>このページはありません。</Alert>;
}
