import { Code, ConnectError } from "@connectrpc/connect";
import { useQuery } from "@connectrpc/connect-query";
import { StaffService } from "@platform/api-client";
import { screen } from "@testing-library/react";

import { renderWithApi } from "../test-utils";

function Me() {
  const me = useQuery(StaffService.method.getMe, {});
  return <p>{me.error ? "失敗" : "…"}</p>;
}

test("トークンが通らなくなったら知らせる(ログインし直してもらう)", async () => {
  const onUnauthenticated = vi.fn<() => void>();
  renderWithApi(
    <Me />,
    ({ service }) => {
      service(StaffService, {
        getMe: () => {
          throw new ConnectError("ログインしてください", Code.Unauthenticated);
        },
      });
    },
    { onUnauthenticated },
  );

  expect(await screen.findByText("失敗")).toBeInTheDocument();
  expect(onUnauthenticated).toHaveBeenCalledOnce();
});

test("入力や権限のエラーは取り直さない(何度送っても同じなので、すぐに出す)", async () => {
  let calls = 0;
  renderWithApi(<Me />, ({ service }) => {
    service(StaffService, {
      getMe: () => {
        calls += 1;
        throw new ConnectError("この操作は管理者だけができます", Code.PermissionDenied);
      },
    });
  });

  expect(await screen.findByText("失敗")).toBeInTheDocument();
  expect(calls).toBe(1);
});
