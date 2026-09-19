import { createRouterTransport, type ConnectRouter } from "@connectrpc/connect";
import { TransportProvider } from "@connectrpc/connect-query";
import { QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import type { ReactNode } from "react";

import { createQueryClient } from "./lib/query";

/// 画面を、テストの中だけのサービス実装につないで描く。
/// gRPC-Web の応答を組み立てる代わりに、Connect のインメモリ transport でサービスを差し替える
export function renderWithApi(
  ui: ReactNode,
  routes: (router: ConnectRouter) => void,
  options: { onUnauthenticated?: () => void } = {},
) {
  const transport = createRouterTransport(routes);
  const queryClient = createQueryClient(options.onUnauthenticated ?? (() => {}));
  return render(
    <TransportProvider transport={transport}>
      <QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>
    </TransportProvider>,
  );
}
