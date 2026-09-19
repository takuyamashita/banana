import { create } from "@bufbuild/protobuf";
import { createClient, createRouterTransport, ConnectError, Code } from "@connectrpc/connect";
import {
  PayrollService,
  PayslipSchema,
  PayslipStatus,
  ProjectService,
  StaffService,
  type ApiClients,
  type Payslip,
} from "@platform/api-client";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ApiProvider } from "../../lib/api";
import { PayslipAdmin } from "./PayslipAdmin";

// gRPC-Web の応答を MSW で組み立てる代わりに、Connect のインメモリ transport でサービスを差し替える。
// 作成・確定・一覧は、このテストの中だけの給与明細の置き場を読み書きする
function clients(options: { rejectCreate?: ConnectError } = {}): ApiClients {
  const payslips: Payslip[] = [];
  const transport = createRouterTransport(({ service }) => {
    service(StaffService, {
      listStaff: () => ({ staff: [{ staffId: 1n, email: "taro@example.com", displayName: "派遣 太郎" }] }),
    });
    service(ProjectService, {
      listProjects: () => ({ projects: [{ projectId: 10n, name: "案件A" }] }),
    });
    service(PayrollService, {
      createPayslip: (req) => {
        if (options.rejectCreate) throw options.rejectCreate;
        const payslipId = BigInt(42 + payslips.length);
        payslips.push(
          create(PayslipSchema, {
            payslipId,
            staffId: req.staffId,
            payYear: req.payYear,
            payMonth: req.payMonth,
            status: PayslipStatus.DRAFT,
            totalYen: 12_000n,
            lines: [{ projectId: 10n, workMinutes: 600, hourlyRate: 1_200n, amountYen: 12_000n }],
          }),
        );
        return { payslipId };
      },
      finalizePayslip: ({ payslipId }) => {
        const p = payslips.find((x) => x.payslipId === payslipId);
        if (p) p.status = PayslipStatus.FINALIZED;
        return {};
      },
      listPayslips: () => ({ payslips }),
    });
  });
  return {
    payroll: createClient(PayrollService, transport),
    staff: createClient(StaffService, transport),
    project: createClient(ProjectService, transport),
  };
}

async function fillAndCreate() {
  const user = userEvent.setup();
  await user.selectOptions(await screen.findByLabelText("派遣社員"), "1");
  await user.clear(screen.getByLabelText("年"));
  await user.type(screen.getByLabelText("年"), "2026");
  await user.clear(screen.getByLabelText("月"));
  await user.type(screen.getByLabelText("月"), "9");
  await user.selectOptions(screen.getByLabelText("案件"), "10");
  await user.type(screen.getByLabelText("稼働(分)"), "600");
  await user.type(screen.getByLabelText("時給(円)"), "1200");
  await user.click(screen.getByRole("button", { name: "作成する" }));
  return user;
}

test("作成すると作成中として一覧に出て、確定すると確定済みになる", async () => {
  render(
    <ApiProvider clients={clients()}>
      <PayslipAdmin />
    </ApiProvider>,
  );
  await screen.findByRole("option", { name: "派遣 太郎(taro@example.com)" });

  const user = await fillAndCreate();

  expect(await screen.findByText(/給与明細 #42 を作成しました。/)).toBeInTheDocument();
  expect(screen.getByText(/作成中/)).toBeInTheDocument();
  expect(screen.getByTestId("payslip-total")).toHaveTextContent("12,000");
  expect(screen.getByRole("cell", { name: "案件A" })).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "派遣 太郎の2026年9月分を確定する" }));

  expect(await screen.findByText("給与明細 #42 を確定しました。")).toBeInTheDocument();
  expect(screen.getByText(/確定済み/)).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "派遣 太郎の2026年9月分を確定する" })).not.toBeInTheDocument();
});

test("サーバーのエラーメッセージを表示する", async () => {
  render(
    <ApiProvider
      clients={clients({ rejectCreate: new ConnectError("この月の給与明細は既にあります", Code.AlreadyExists) })}
    >
      <PayslipAdmin />
    </ApiProvider>,
  );
  await screen.findByRole("option", { name: "派遣 太郎(taro@example.com)" });

  await fillAndCreate();

  expect(await screen.findByRole("alert")).toHaveTextContent("この月の給与明細は既にあります");
});

test("派遣社員を選び直すと、新しい一覧が届くまで前の人の給与明細と確定ボタンを出さない", async () => {
  // 花子の一覧は、テストが release を呼ぶまで返さない
  let release = () => {};
  const hanakoListed = new Promise<void>((resolve) => {
    release = resolve;
  });
  const draft = (staffId: bigint, payslipId: bigint) =>
    create(PayslipSchema, {
      payslipId,
      staffId,
      payYear: 2026,
      payMonth: 9,
      status: PayslipStatus.DRAFT,
      totalYen: 12_000n,
      lines: [{ projectId: 10n, workMinutes: 600, hourlyRate: 1_200n, amountYen: 12_000n }],
    });
  const transport = createRouterTransport(({ service }) => {
    service(StaffService, {
      listStaff: () => ({
        staff: [
          { staffId: 1n, email: "taro@example.com", displayName: "派遣 太郎" },
          { staffId: 2n, email: "hanako@example.com", displayName: "派遣 花子" },
        ],
      }),
    });
    service(ProjectService, { listProjects: () => ({ projects: [{ projectId: 10n, name: "案件A" }] }) });
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
  render(
    <ApiProvider
      clients={{
        payroll: createClient(PayrollService, transport),
        staff: createClient(StaffService, transport),
        project: createClient(ProjectService, transport),
      }}
    >
      <PayslipAdmin />
    </ApiProvider>,
  );
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
