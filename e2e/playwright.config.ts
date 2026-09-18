import { defineConfig, devices } from "@playwright/test";

// 前提: docker compose の依存サービスとマイグレーション済みの DB(mise run e2e が用意する)
export default defineConfig({
  testDir: "./tests",
  fullyParallel: true,
  retries: process.env["CI"] ? 1 : 0,
  reporter: process.env["CI"] ? "github" : "list",
  use: {
    baseURL: "http://localhost:5173",
    trace: "retain-on-failure",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  // 起動済みならそれを使う。server は /health が 200 になるまで待つ
  webServer: [
    {
      command: "cargo run -p server",
      url: "http://localhost:50051/health",
      reuseExistingServer: true,
      cwd: "..",
      timeout: 300_000,
    },
    {
      command: "pnpm --filter web dev",
      url: "http://localhost:5173",
      reuseExistingServer: true,
      cwd: "..",
    },
  ],
});
