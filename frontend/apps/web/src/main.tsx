import "@platform/ui/styles.css";

import { Alert, pageClass } from "@platform/ui";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App, createApp } from "./app";
import { loadRuntimeConfig } from "./lib/config";

const root = document.getElementById("root");
if (!root) throw new Error("#root not found");

// 接続先は実行時に /config.json から読む。ビルド成果物を全環境で同一にし、
// デプロイ時に環境ごとの config.json を置くだけで切り替える。読めなければ白い画面にせず、そう伝える
try {
  const app = await createApp(await loadRuntimeConfig());
  createRoot(root).render(
    <StrictMode>
      <App {...app} />
    </StrictMode>,
  );
} catch (err) {
  createRoot(root).render(
    <main className={pageClass()}>
      <h1>給与管理</h1>
      <Alert>画面を開けませんでした。{err instanceof Error ? err.message : ""}管理者に連絡してください。</Alert>
    </main>,
  );
}
