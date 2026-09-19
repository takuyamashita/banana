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
  // 録画にはマウスカーソルが映らないので、カーソルの形をした要素を画面に足す(ログインの IdP の画面にも出す)
  await context.addInitScript(showCursor);
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

/// カーソルの代わりの矢印を画面に出し、マウスの動きに合わせて動かす。押した場所には波紋を出す。
/// fill・selectOption はマウスを動かさないので、入力欄に入ったときもその欄へ動かす。
/// ブラウザの中で動く(addInitScript に渡す)ので、外の変数は使わない
function showCursor() {
  const install = () => {
    if (document.getElementById("__video-cursor")) return;
    const svgNs = "http://www.w3.org/2000/svg";
    const svg = document.createElementNS(svgNs, "svg");
    svg.setAttribute("width", "24");
    svg.setAttribute("height", "24");
    svg.setAttribute("viewBox", "0 0 24 24");
    svg.id = "__video-cursor";
    const arrow = document.createElementNS(svgNs, "path");
    arrow.setAttribute("d", "M3 2 L3 19 L8 14.5 L11.5 22 L14.5 20.6 L11 13.4 L17.5 13.4 Z");
    arrow.setAttribute("fill", "#111");
    arrow.setAttribute("stroke", "#fff");
    arrow.setAttribute("stroke-width", "1.5");
    svg.append(arrow);
    Object.assign(svg.style, {
      position: "fixed",
      left: "0",
      top: "0",
      zIndex: "2147483647",
      pointerEvents: "none",
      transition: "transform 150ms ease-out",
      filter: "drop-shadow(0 1px 2px rgb(0 0 0 / 40%))",
    });
    document.documentElement.append(svg);

    const move = (x: number, y: number) => {
      svg.style.transform = `translate(${x - 3}px, ${y - 2}px)`;
    };
    // 画面を移っても、前の画面で最後にいた場所から出す(はじめは画面の真ん中)
    const saved = /^(\d+),(\d+)$/.exec(window.name);
    move(saved ? Number(saved[1]) : innerWidth / 2, saved ? Number(saved[2]) : innerHeight / 2);
    const remember = (x: number, y: number) => {
      move(x, y);
      window.name = `${Math.round(x)},${Math.round(y)}`;
    };

    addEventListener("mousemove", (e) => remember(e.clientX, e.clientY), true);
    addEventListener(
      "mousedown",
      (e) => {
        const ripple = document.createElement("div");
        Object.assign(ripple.style, {
          position: "fixed",
          left: `${e.clientX - 18}px`,
          top: `${e.clientY - 18}px`,
          width: "36px",
          height: "36px",
          borderRadius: "50%",
          border: "3px solid rgb(255 80 80 / 90%)",
          zIndex: "2147483646",
          pointerEvents: "none",
          transition: "transform 400ms ease-out, opacity 400ms ease-out",
        });
        document.documentElement.append(ripple);
        requestAnimationFrame(() => {
          ripple.style.transform = "scale(1.8)";
          ripple.style.opacity = "0";
        });
        setTimeout(() => ripple.remove(), 500);
      },
      true,
    );
    addEventListener(
      "focusin",
      (e) => {
        const target = e.target;
        if (!(target instanceof HTMLInputElement || target instanceof HTMLSelectElement)) return;
        const rect = target.getBoundingClientRect();
        remember(rect.left + Math.min(rect.width / 2, 48), rect.top + rect.height / 2);
      },
      true,
    );
  };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", install);
  else install();
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
