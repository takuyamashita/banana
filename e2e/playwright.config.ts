import { defineConfig, devices } from "@playwright/test";

// 前提: docker compose の依存サービスとマイグレーション済みの DB(mise run e2e が用意する)
// ポートは環境変数で変えられる(worktree ごとに別の組を立てるとき、mise が .env.worktree から渡す)
const WEB = `http://localhost:${process.env["WEB_PORT"] ?? 5173}`;
const API = `http://localhost:${process.env["API_PORT"] ?? 50051}`;

export default defineConfig({
  testDir: "./tests",
  fullyParallel: true,
  retries: process.env["CI"] ? 1 : 0,
  // ローカルは HTML レポートも出す(mise run e2e:report で、失敗時のトレースと一緒に見られる)
  reporter: process.env["CI"] ? "github" : [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: WEB,
    trace: "retain-on-failure",
    // mise run e2e:headed で動きを目で追えるように、操作の間を空ける(既定は 0)
    launchOptions: { slowMo: Number(process.env["E2E_SLOW_MO"] ?? 0) },
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  // 起動済みならそれを使う。server は /health が 200 になるまで待つ
  webServer: [
    {
      command: "cargo run -p server",
      url: `${API}/health`,
      reuseExistingServer: true,
      cwd: "..",
      timeout: 300_000,
    },
    {
      command: "pnpm --filter web dev",
      url: WEB,
      reuseExistingServer: true,
      cwd: "..",
    },
  ],
});
