import type { Transport } from "@platform/api-client";
import type { QueryClient } from "@tanstack/react-query";
import { createRouter } from "@tanstack/react-router";

import { RouteError, RoutePending } from "./layout/RouteStatus";
import type { Auth } from "./lib/auth";
import { routeTree } from "./routeTree.gen";

/// どのルートからも使えるもの。loader と beforeLoad は、これを通して API とログインの状態に触れる
export interface RouterContext {
  auth: Auth;
  queryClient: QueryClient;
  transport: Transport;
}

export function createAppRouter(context: RouterContext) {
  return createRouter({
    routeTree,
    context,
    // データの置き場は TanStack Query に任せる。ルーターの側では loader の結果を古いものとして扱い、
    // 画面を開くたびに loader を通す(中身は Query のキャッシュから返る)
    defaultPreloadStaleTime: 0,
    defaultPendingComponent: RoutePending,
    defaultErrorComponent: RouteError,
    // URL の引数は文字列のまま受け渡し、読むのは各ルートの validateSearch(スキーマ)に任せる。
    // 既定の JSON 形式だと、番号のような文字列が ?staffId=%221%22 と引用符付きになる
    parseSearch: (search) => Object.fromEntries(new URLSearchParams(search)),
    stringifySearch: (search) => {
      const entries = Object.entries(search).flatMap(([key, value]) =>
        typeof value === "string" ? [[key, value]] : [],
      );
      const query = new URLSearchParams(entries).toString();
      return query === "" ? "" : `?${query}`;
    },
  });
}

export type AppRouter = ReturnType<typeof createAppRouter>;

declare module "@tanstack/react-router" {
  interface Register {
    router: AppRouter;
  }
}
