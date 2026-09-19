// 動作確認の動画を撮る設定(mise run e2e:video)。依存サービス・server・Vite は普段の E2E と同じものを使う
import { defineConfig } from "@playwright/test";

import base from "./playwright.config";
import { VIDEO_SIZE } from "./support/video";

export default defineConfig({
  ...base,
  // 台本は確かめるテスト(tests/)と分け、普段の mise run e2e では流さない
  testDir: "./videos",
  outputDir: "./test-results/videos",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 180_000,
  reporter: [["list"]],
  use: {
    ...base.use,
    trace: "off",
    viewport: VIDEO_SIZE,
    // 目で追えるように、操作の間を空ける
    launchOptions: { slowMo: Number(process.env["E2E_SLOW_MO"] ?? 250) },
  },
  projects: [{ name: "video" }],
});
