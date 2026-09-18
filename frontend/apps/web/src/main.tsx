import "@platform/ui/styles.css";
import "./app.css";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./App";
import { loadRuntimeConfig } from "./lib/config";

// 接続先は実行時に /config.json から読む。ビルド成果物を全環境で同一にし、
// デプロイ時に環境ごとの config.json を置くだけで切り替える
const config = await loadRuntimeConfig();

const root = document.getElementById("root");
if (!root) throw new Error("#root not found");

createRoot(root).render(
  <StrictMode>
    <App config={config} />
  </StrictMode>,
);
