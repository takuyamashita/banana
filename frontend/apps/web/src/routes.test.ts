import type { AnyRoute } from "@tanstack/react-router";

import { routeTree } from "./routeTree.gen";

/// ルートの下のルートをすべて並べる
function descendants(route: AnyRoute): AnyRoute[] {
  const children: AnyRoute[] = Object.values(route.children ?? {});
  return children.flatMap((child) => [child, ...descendants(child)]);
}

test("ログインした人の画面は、どれもどのシステム(給与・勤怠)のものかを決めている", () => {
  const app = descendants(routeTree).find((route) => "id" in route.options && route.options.id === "/_app");
  const pages = descendants(app ?? routeTree).filter((route) => route.options.component);

  expect(pages.length).toBeGreaterThan(0);
  for (const page of pages) {
    const path: unknown = "path" in page.options ? page.options.path : undefined;
    expect({ path, system: page.options.staticData?.system }).toEqual({
      path,
      system: expect.stringMatching(/^(payroll|timesheet)$/),
    });
  }
});
