import { create } from "@bufbuild/protobuf";
import { createClient, createRouterTransport, ConnectError, Code } from "@connectrpc/connect";
import {
  PayrollService,
  PayslipSchema,
  PayslipStatus,
  ProjectService,
  StaffService,
  type ApiClients,
} from "@platform/api-client";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ApiProvider } from "../../lib/api";
import { FinalizePayslipForm } from "./FinalizePayslipForm";

// gRPC-Web の応答を MSW で組み立てる代わりに、Connect のインメモリ transport でサービスを差し替える
function clients(finalize: () => bigint): ApiClients {
  const transport = createRouterTransport(({ service }) => {
    service(StaffService, {
      listStaff: () => ({ staff: [{ staffId: 1n, email: "taro@example.com", displayName: "派遣 太郎" }] }),
    });
    service(ProjectService, {
      listProjects: () => ({ projects: [{ projectId: 10n, name: "案件A" }] }),
    });
    service(PayrollService, {
      finalizePayslip: () => ({ payslipId: finalize() }),
      getPayslip: ({ payslipId }) => ({
        payslip: create(PayslipSchema, {
          payslipId,
          staffId: 1n,
          payYear: 2026,
          payMonth: 9,
          status: PayslipStatus.FINALIZED,
          totalYen: 12_000n,
          lines: [{ projectId: 10n, workMinutes: 600, hourlyRate: 1_200n, amountYen: 12_000n }],
        }),
      }),
    });
  });
  return {
    payroll: createClient(PayrollService, transport),
    staff: createClient(StaffService, transport),
    project: createClient(ProjectService, transport),
  };
}

async function fillAndSubmit() {
  const user = userEvent.setup();
  await user.selectOptions(await screen.findByLabelText("派遣社員"), "1");
  await user.clear(screen.getByLabelText("年"));
  await user.type(screen.getByLabelText("年"), "2026");
  await user.clear(screen.getByLabelText("月"));
  await user.type(screen.getByLabelText("月"), "9");
  await user.selectOptions(screen.getByLabelText("案件"), "10");
  await user.type(screen.getByLabelText("稼働(分)"), "600");
  await user.type(screen.getByLabelText("時給(円)"), "1200");
  await user.click(screen.getByRole("button", { name: "確定する" }));
}

test("確定すると明細と合計が表示される", async () => {
  render(
    <ApiProvider clients={clients(() => 42n)}>
      <FinalizePayslipForm />
    </ApiProvider>,
  );
  await screen.findByRole("option", { name: "派遣 太郎(taro@example.com)" });

  await fillAndSubmit();

  expect(await screen.findByText("給与明細 #42 を確定しました。")).toBeInTheDocument();
  expect(screen.getByTestId("payslip-total")).toHaveTextContent("12,000");
  expect(screen.getByRole("cell", { name: "案件A" })).toBeInTheDocument();
});

test("サーバーのエラーメッセージを表示する", async () => {
  const conflict = () => {
    throw new ConnectError("この月の給与明細は既に確定しています", Code.AlreadyExists);
  };
  render(
    <ApiProvider clients={clients(conflict)}>
      <FinalizePayslipForm />
    </ApiProvider>,
  );
  await screen.findByRole("option", { name: "派遣 太郎(taro@example.com)" });

  await fillAndSubmit();

  expect(await screen.findByRole("alert")).toHaveTextContent("この月の給与明細は既に確定しています");
});
