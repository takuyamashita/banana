import { TransportProvider } from "@connectrpc/connect-query";
import { createApiTransport, type Transport } from "@platform/api-client";
import { QueryClientProvider, type QueryClient } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";

import { createAuth, createUserManager } from "./lib/auth";
import type { RuntimeConfig } from "./lib/config";
import { SESSION_EXPIRED } from "./lib/errors";
import { createQueryClient } from "./lib/query";
import { createAppRouter, type AppRouter } from "./router";

/// アプリの初期化。画面を描く前に1回だけ行う(Effect の中で初期化しない)
export async function createApp(config: RuntimeConfig): Promise<{
  router: AppRouter;
  queryClient: QueryClient;
  transport: Transport;
}> {
  const manager = createUserManager(config);
  const holder: { router?: AppRouter; queryClient?: QueryClient } = {};
  // ログインが切れたら、前の利用者で取ったデータを捨て、ログイン画面に移る(戻り先は今の画面)。
  // 移る間に画面が取り直して、また切れたと知らされても、ログイン画面を戻り先にはしない
  const auth = createAuth(manager, config, () => {
    holder.queryClient?.clear();
    const router = holder.router;
    if (!router || router.state.location.pathname === "/login") return;
    void router.navigate({ to: "/login", search: { returnTo: router.state.location.href } });
  });
  const queryClient = createQueryClient(() => auth.expire(SESSION_EXPIRED));
  const transport = createApiTransport({ baseUrl: config.apiBaseUrl, getAccessToken: auth.accessToken });
  // ログインからの戻りを、ルーターが URL を読む前に処理する(URL が戻り先の画面になる)
  await auth.restore();
  const router = createAppRouter({ auth, queryClient, transport });
  Object.assign(holder, { router, queryClient });
  return { router, queryClient, transport };
}

export function App({ router, queryClient, transport }: Awaited<ReturnType<typeof createApp>>) {
  return (
    <TransportProvider transport={transport}>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </TransportProvider>
  );
}
