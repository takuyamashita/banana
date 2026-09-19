// 動作確認の動画(mise run e2e:video)を撮るための補助。確かめるテストでは使わない
import { join } from "node:path";

import type { Browser, Page } from "@playwright/test";

/// 動画の出力先(git には入れない)
export const VIDEO_DIR = join(import.meta.dirname, "..", "videos-out");
export const VIDEO_SIZE = { width: 1280, height: 800 };

/// 録画するブラウザを1つ開く。close() で閉じて <name>.webm に保存する(mp4 への変換はタスクが行う)
export async function recordPage(browser: Browser, name: string): Promise<{ page: Page; close: () => Promise<void> }> {
  const context = await browser.newContext({
    viewport: VIDEO_SIZE,
    recordVideo: { dir: join(VIDEO_DIR, "raw"), size: VIDEO_SIZE },
  });
  const page = await context.newPage();
  return {
    page,
    close: async () => {
      await context.close();
      const video = page.video();
      if (!video) throw new Error("録画されていない");
      await video.saveAs(join(VIDEO_DIR, `${name}.webm`));
      await video.delete();
    },
  };
}

/// 画面の下に、いま何を確かめているかの字幕を出し、読める間だけ待つ。
/// 字幕はテストが画面に足すだけで、アプリのコードには入れない(画面を移ると消えるので、移った後に出し直す)
export async function caption(page: Page, text: string, holdMs = 2500): Promise<void> {
  await page.evaluate((t) => {
    let el = document.getElementById("__video-caption");
    if (!el) {
      el = document.createElement("div");
      el.id = "__video-caption";
      Object.assign(el.style, {
        position: "fixed",
        left: "50%",
        bottom: "24px",
        transform: "translateX(-50%)",
        zIndex: "2147483647",
        maxWidth: "90%",
        padding: "10px 18px",
        borderRadius: "8px",
        background: "rgb(0 0 0 / 80%)",
        color: "#fff",
        font: "600 18px system-ui, 'Noto Sans JP', sans-serif",
      });
      document.body.append(el);
    }
    el.textContent = t;
  }, text);
  await page.waitForTimeout(holdMs);
}
