import type { RouterContext } from "./lib/context";
import { createAppRouter } from "./router";

test("ログインした人の画面は、どれもどのシステム(給与・勤怠)のものかを決めている", () => {
  // ルートの ID(/_app/me など)は、ルーターを作るときに決まる。画面は開かないので、中身の要る context は渡さない
  const { routesById } = createAppRouter({} as RouterContext);
  const pages = Object.values(routesById).filter((route) => route.id.startsWith("/_app/") && route.options.component);

  expect(pages.length).toBeGreaterThan(0);
  for (const page of pages) {
    expect({ id: page.id, system: page.options.staticData?.system }).toEqual({
      id: page.id,
      system: expect.stringMatching(/^(payroll|timesheet)$/),
    });
  }
});
