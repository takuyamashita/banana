// 利用者ごとに別のブラウザ(コンテキスト)を開くためのフィクスチャ。開いたものはテストの終わりに閉じる
import { test as base, type BrowserContext, type Page } from "@playwright/test";

export const test = base.extend<{ newPage: () => Promise<Page> }>({
  newPage: async ({ browser }, use) => {
    const contexts: BrowserContext[] = [];
    await use(async () => {
      const context = await browser.newContext();
      contexts.push(context);
      return context.newPage();
    });
    await Promise.all(contexts.map((c) => c.close()));
  },
});

export { expect } from "@playwright/test";
