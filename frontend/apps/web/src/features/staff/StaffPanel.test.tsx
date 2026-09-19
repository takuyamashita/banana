import { Code, ConnectError } from "@connectrpc/connect";
import { StaffService, type Staff } from "@platform/api-client";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { renderWithApi } from "../../test-utils";
import { StaffPanel } from "./StaffPanel";

async function register(user: ReturnType<typeof userEvent.setup>) {
  await user.type(screen.getByLabelText("メールアドレス"), "hanako@example.com");
  await user.type(screen.getByLabelText("表示名"), "派遣 花子");
  await user.type(screen.getByLabelText("仮パスワード"), "Temp-Passw0rd!");
  await user.click(screen.getByRole("button", { name: "登録する" }));
}

test("登録すると一覧に加わり、入力欄は空に戻る", async () => {
  const staff: Omit<Staff, "$typeName">[] = [];
  renderWithApi(<StaffPanel />, ({ service }) => {
    service(StaffService, {
      listStaff: () => ({ staff }),
      createStaff: (req) => {
        staff.push({ staffId: 3n, email: req.email, displayName: req.displayName });
        return { staffId: 3n };
      },
    });
  });
  const user = userEvent.setup();

  await register(user);

  expect(await screen.findByText(/派遣社員 #3 を登録しました。/)).toBeInTheDocument();
  expect(await screen.findByRole("cell", { name: "派遣 花子" })).toBeInTheDocument();
  expect(screen.getByLabelText("メールアドレス")).toHaveValue("");
});

test("登録できなかったらサーバーの文言を出し、入力は残す", async () => {
  renderWithApi(<StaffPanel />, ({ service }) => {
    service(StaffService, {
      listStaff: () => ({ staff: [] }),
      createStaff: () => {
        throw new ConnectError("このメールアドレスは既に登録されています", Code.AlreadyExists);
      },
    });
  });
  const user = userEvent.setup();

  await register(user);

  expect(await screen.findByRole("alert")).toHaveTextContent("このメールアドレスは既に登録されています");
  expect(screen.getByLabelText("メールアドレス")).toHaveValue("hanako@example.com");
});
