import { create } from "@bufbuild/protobuf";
import { Code, ConnectError, type ConnectRouter } from "@connectrpc/connect";
import {
  PayrollService,
  PayslipSchema,
  PayslipStatus,
  ProjectService,
  StaffService,
  type CreatePayslipRequest,
  type Payslip,
} from "@platform/api-client";
import { screen, within } from "@testing-library/react";
import { useState } from "react";
import userEvent from "@testing-library/user-event";

import { renderWithApi } from "../../test-utils";
import { PayslipAdminPage } from "./PayslipAdmin";

const taro = { staffId: 1n, email: "taro@example.com", displayName: "派遣 太郎" };
const hanako = { staffId: 2n, email: "hanako@example.com", displayName: "派遣 花子" };
const projects = [
  { projectId: 10n, name: "案件A" },
  { projectId: 11n, name: "案件B" },
];

const draft = (staffId: bigint, payslipId: bigint) =>
  create(PayslipSchema, {
    payslipId,
    staffId,
    payYear: 2026,
    payMonth: 9,
    status: PayslipStatus.DRAFT,
    totalYen: 12_000n,
    lines: [{ projectId: 10n, projectName: "案件A", workMinutes: 600, hourlyRate: 1_200n, amountYen: 12_000n }],
  });

/// 作成・確定・一覧は、このテストの中だけの給与明細の置き場を読み書きする
function routes(
  options: { rejectCreate?: ConnectError; received?: CreatePayslipRequest[]; hold?: Promise<void> } = {},
) {
  const payslips: Payslip[] = [];
  return ({ service }: ConnectRouter) => {
    service(StaffService, { listStaff: () => ({ staff: [taro] }) });
    service(ProjectService, { listProjects: () => ({ projects }) });
    service(PayrollService, {
      createPayslip: async (req) => {
        options.received?.push(req);
        await options.hold;
        if (options.rejectCreate) throw options.rejectCreate;
        const payslip = draft(req.staffId, BigInt(42 + payslips.length));
        payslips.push(payslip);
        return { payslipId: payslip.payslipId };
      },
      finalizePayslip: ({ payslipId }) => {
        const p = payslips.find((x) => x.payslipId === payslipId);
        if (p) p.status = PayslipStatus.FINALIZED;
        return {};
      },
      listPayslips: () => ({ payslips }),
    });
  };
}

/// URL の代わりに、選んでいる派遣社員を state で持つ(本番はルートが URL の引数から渡す)
function Page() {
  const [staffId, setStaffId] = useState<string | undefined>(undefined);
  return <PayslipAdminPage staffId={staffId} defaultPeriod={{ year: 2026, month: 8 }} onSelectStaff={setStaffId} />;
}

async function fillLine(user: ReturnType<typeof userEvent.setup>, minutes: string) {
  await user.selectOptions(await screen.findByLabelText("派遣社員"), "1");
  await user.clear(screen.getByLabelText("年"));
  await user.type(screen.getByLabelText("年"), "2026");
  await user.clear(screen.getByLabelText("月"));
  await user.type(screen.getByLabelText("月"), "9");
  await user.selectOptions(screen.getByLabelText("案件"), "10");
  await user.type(screen.getByLabelText("稼働(分)"), minutes);
  await user.type(screen.getByLabelText("時給(円)"), "1200");
}

async function renderReady(r: ReturnType<typeof routes>) {
  renderWithApi(<Page />, r);
  await screen.findByRole("option", { name: "派遣 太郎(taro@example.com)" });
  await screen.findByRole("option", { name: "案件A" });
}

test("作成すると作成中として一覧に出て、確定すると確定済みになる", async () => {
  await renderReady(routes());
  const user = userEvent.setup();

  await fillLine(user, "600");
  await user.click(screen.getByRole("button", { name: "作成する" }));

  expect(await screen.findByText(/給与明細 #42 を作成しました。/)).toBeInTheDocument();
  expect(await screen.findByText(/作成中/)).toBeInTheDocument();
  expect(screen.getByTestId("payslip-total")).toHaveTextContent("12,000");
  expect(screen.getByRole("cell", { name: "案件A" })).toBeInTheDocument();
  expect(screen.getByRole("cell", { name: "10時間" })).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "派遣 太郎の2026年9月分を確定する" }));

  expect(await screen.findByText("給与明細 #42 を確定しました。")).toBeInTheDocument();
  expect(await screen.findByText(/確定済み/)).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /確定する/ })).not.toBeInTheDocument();
});

test("サーバーのエラーメッセージを表示する", async () => {
  await renderReady(routes({ rejectCreate: new ConnectError("この月の給与明細は既にあります", Code.AlreadyExists) }));
  const user = userEvent.setup();

  await fillLine(user, "600");
  await user.click(screen.getByRole("button", { name: "作成する" }));

  expect(await screen.findByRole("alert")).toHaveTextContent("この月の給与明細は既にあります");
});

test("数字として読めない入力は送らずに、どこが違うかを伝える", async () => {
  const received: CreatePayslipRequest[] = [];
  await renderReady(routes({ received }));
  const user = userEvent.setup();

  await fillLine(user, "1.5");
  await user.click(screen.getByRole("button", { name: "作成する" }));

  expect(await screen.findByRole("alert")).toHaveTextContent("明細 1の稼働(分)は数字で入力してください。");
  expect(received).toHaveLength(0);
});

test("送っている間は作成ボタンを押せない(同じ給与明細を二重に作らない)", async () => {
  let release = () => {};
  const hold = new Promise<void>((resolve) => {
    release = resolve;
  });
  const received: CreatePayslipRequest[] = [];
  await renderReady(routes({ received, hold }));
  const user = userEvent.setup();

  await fillLine(user, "600");
  await user.click(screen.getByRole("button", { name: "作成する" }));

  expect(screen.getByRole("button", { name: "作成する" })).toBeDisabled();
  await user.click(screen.getByRole("button", { name: "作成する" }));
  release();
  expect(await screen.findByText(/給与明細 #42 を作成しました。/)).toBeInTheDocument();
  expect(received).toHaveLength(1);
});

test("明細行を消しても、残りの行の入力はそのまま", async () => {
  await renderReady(routes());
  const user = userEvent.setup();

  await user.click(screen.getByRole("button", { name: "明細を追加" }));
  const [first, second] = screen.getAllByRole("group");
  await user.type(within(first!).getByLabelText("稼働(分)"), "60");
  await user.type(within(second!).getByLabelText("稼働(分)"), "120");

  await user.click(screen.getByRole("button", { name: "明細 1 を削除" }));

  const rest = screen.getAllByRole("group");
  expect(rest).toHaveLength(1);
  expect(within(rest[0]!).getByLabelText("稼働(分)")).toHaveValue("120");
  // 番号は振り直す
  expect(rest[0]).toHaveAccessibleName("明細 1");
});

test("派遣社員を選び直すと、新しい一覧が届くまで前の人の給与明細と確定ボタンを出さない", async () => {
  // 花子の一覧は、テストが release を呼ぶまで返さない
  let release = () => {};
  const hanakoListed = new Promise<void>((resolve) => {
    release = resolve;
  });
  renderWithApi(<Page />, ({ service }) => {
    service(StaffService, { listStaff: () => ({ staff: [taro, hanako] }) });
    service(ProjectService, { listProjects: () => ({ projects }) });
    service(PayrollService, {
      listPayslips: async ({ staffId }) => {
        if (staffId === 2n) {
          await hanakoListed;
          return { payslips: [draft(2n, 7n)] };
        }
        return { payslips: [draft(1n, 5n)] };
      },
    });
  });
  const user = userEvent.setup();
  const select = await screen.findByLabelText("派遣社員");
  await screen.findByRole("option", { name: "派遣 花子(hanako@example.com)" });

  await user.selectOptions(select, "1");
  expect(await screen.findByRole("button", { name: "派遣 太郎の2026年9月分を確定する" })).toBeInTheDocument();

  await user.selectOptions(select, "2");
  expect(screen.getByRole("status")).toHaveTextContent("読み込み中");
  expect(screen.queryByRole("button", { name: /確定する/ })).not.toBeInTheDocument();

  release();
  expect(await screen.findByRole("button", { name: "派遣 花子の2026年9月分を確定する" })).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /派遣 太郎/ })).not.toBeInTheDocument();
});
