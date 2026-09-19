import react from "@vitejs/plugin-react";
import { loadEnv, type Plugin } from "vite";
import { defineConfig } from "vitest/config";

// ポートは環境変数で変えられる(worktree ごとに別の組を立てるとき、mise が .env.worktree から渡す)
const env = loadEnv("development", ".", "");
const port = (name: string, fallback: number) => Number(env[name] ?? fallback);

// 開発サーバーでは /config.json を public/ のファイルではなく、ポートの環境変数から組み立てて返す
const devRuntimeConfig = (): Plugin => ({
  name: "dev-runtime-config",
  apply: "serve",
  configureServer(server) {
    server.middlewares.use("/config.json", (_req, res) => {
      res.setHeader("content-type", "application/json");
      res.end(
        JSON.stringify({
          apiBaseUrl: `http://localhost:${port("API_PORT", 50051)}`,
          oidc: {
            authority: `http://localhost:${port("KEYCLOAK_PORT", 8080)}/realms/platform`,
            clientId: "web",
          },
        }),
      );
    });
  },
});

export default defineConfig({
  plugins: [react(), devRuntimeConfig()],
  server: { port: port("WEB_PORT", 5173), strictPort: true },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test-setup.ts"],
  },
});
