import type { DescMessage, DescMethodUnary, MessageInitShape, MessageShape } from "@bufbuild/protobuf";
import { createQueryOptions } from "@connectrpc/connect-query";
import { Code, ConnectError } from "@platform/api-client";
import { redirect } from "@tanstack/react-router";

import type { RouterContext } from "./context";

/// loader・beforeLoad で、画面が使うデータを先に取っておく。画面は同じメソッドと入力の
/// useSuspenseQuery で、取っておいたものを受け取る。
/// ログインが切れていたら、ログインし直してもらう(戻り先は今開こうとしている画面)
export async function ensure<I extends DescMessage, O extends DescMessage>(
  context: RouterContext,
  method: DescMethodUnary<I, O>,
  input: MessageInitShape<I>,
  returnTo: string,
): Promise<MessageShape<O>> {
  try {
    return await context.queryClient.ensureQueryData(
      createQueryOptions(method, input, { transport: context.transport }),
    );
  } catch (err) {
    if (err instanceof ConnectError && err.code === Code.Unauthenticated) {
      throw redirect({ to: "/login", search: { returnTo } });
    }
    throw err;
  }
}
