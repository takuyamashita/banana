import react from "@vitejs/plugin-react";
import { loadEnv, type Connect, type Plugin } from "vite";
import { defineConfig } from "vitest/config";

// ポートは環境変数で変えられる(worktree ごとに別の組を立てるとき、mise が worktree の .env から渡す)
const env = loadEnv("development", ".", "");
const port = (name: string, fallback: number) => {
  const value = Number(env[name] || fallback);
  if (!Number.isInteger(value) || value <= 0) throw new Error(`${name} がポート番号ではありません: ${env[name]}`);
  return value;
};

// 開発サーバーと vite preview では /config.json を public/ のファイルではなく、ポートの環境変数から組み立てて返す
const localRuntimeConfig = (): Plugin => {
  const body = JSON.stringify({
    apiBaseUrl: `http://localhost:${port("API_PORT", 50051)}`,
    oidc: {
      authority: `http://localhost:${port("KEYCLOAK_PORT", 8080)}/realms/platform`,
      clientId: "web",
    },
  });
  const serve = (server: { middlewares: Connect.Server }) => {
    server.middlewares.use("/config.json", (_req, res) => {
      res.setHeader("content-type", "application/json");
      res.end(body);
    });
  };
  return { name: "local-runtime-config", configureServer: serve, configurePreviewServer: serve };
};

export default defineConfig({
  plugins: [react(), localRuntimeConfig()],
  server: { port: port("WEB_PORT", 5173), strictPort: true },
  preview: { port: port("WEB_PORT", 5173), strictPort: true },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test-setup.ts"],
  },
});
