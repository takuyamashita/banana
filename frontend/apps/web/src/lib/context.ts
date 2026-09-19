import type { Transport } from "@platform/api-client";
import type { QueryClient } from "@tanstack/react-query";

import type { Auth } from "./auth";

/// どのルートからも使えるもの。loader と beforeLoad は、これを通して API とログインの状態に触れる
export interface RouterContext {
  auth: Auth;
  queryClient: QueryClient;
  transport: Transport;
}
