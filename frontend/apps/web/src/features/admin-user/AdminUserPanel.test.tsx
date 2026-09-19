import { Code, ConnectError } from "@connectrpc/connect";
import { UserService } from "@platform/api-client";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { renderWithApi } from "../../test-utils";
import { AdminUserPage } from "./AdminUserPanel";

async function add(user: ReturnType<typeof userEvent.setup>) {
  await user.type(await screen.findByLabelText("メールアドレス"), "new-admin@example.com");
  await user.type(screen.getByLabelText("仮パスワード"), "Temp-Passw0rd!");
  await user.click(screen.getByRole("button", { name: "追加する" }));
}

test("追加すると、どのメールアドレスを管理者にしたかが出て、入力欄は空に戻る", async () => {
  renderWithApi(<AdminUserPage />, ({ service }) => {
    service(UserService, {
      createAdminUser: (req) => ({ userId: `sub-${req.email}` }),
    });
  });
  const user = userEvent.setup();

  await add(user);

  expect(await screen.findByText(/new-admin@example.com を管理者にしました。/)).toBeInTheDocument();
  expect(screen.getByLabelText("メールアドレス")).toHaveValue("");
  expect(screen.getByLabelText("仮パスワード")).toHaveValue("");
});

test("追加できなかったらサーバーの文言を出し、入力は残す", async () => {
  renderWithApi(<AdminUserPage />, ({ service }) => {
    service(UserService, {
      createAdminUser: () => {
        throw new ConnectError("同じメールアドレスの利用者が既にいます", Code.AlreadyExists);
      },
    });
  });
  const user = userEvent.setup();

  await add(user);

  expect(await screen.findByRole("alert")).toHaveTextContent("同じメールアドレスの利用者が既にいます");
  expect(screen.getByLabelText("メールアドレス")).toHaveValue("new-admin@example.com");
});
