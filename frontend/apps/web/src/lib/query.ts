import { Code, ConnectError } from "@platform/api-client";
import { MutationCache, QueryCache, QueryClient } from "@tanstack/react-query";

import { isTransient } from "./errors";

/// 画面が API から取ったデータの置き場。
///
/// どの取得・更新でもトークンが通らなくなったら `onUnauthenticated` を呼ぶ(ログインし直してもらう)。
/// 取り直すのは時間をおけば通るかもしれないエラーだけ。入力の誤りや権限のエラーは何度送っても同じなので、すぐに出す
export function createQueryClient(onUnauthenticated: () => void): QueryClient {
  const onError = (err: unknown) => {
    if (err instanceof ConnectError && err.code === Code.Unauthenticated) onUnauthenticated();
  };
  return new QueryClient({
    queryCache: new QueryCache({ onError }),
    mutationCache: new MutationCache({ onError }),
    defaultOptions: {
      queries: {
        retry: (failures, err) => isTransient(err) && failures < 2,
        // 別のタブから戻るたびに取り直さない(一覧は更新したときに取り直す)
        refetchOnWindowFocus: false,
      },
      mutations: { retry: false },
    },
  });
}
