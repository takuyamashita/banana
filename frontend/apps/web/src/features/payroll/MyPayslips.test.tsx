import { create } from "@bufbuild/protobuf";
import { PayrollService, PayslipSchema, PayslipStatus } from "@platform/api-client";
import { screen } from "@testing-library/react";

import { renderWithApi } from "../../test-utils";
import { MyPayslipsPage } from "./MyPayslips";

test("確定した給与明細がまだなければ、そう伝える", async () => {
  renderWithApi(<MyPayslipsPage staffId={1n} />, ({ service }) => {
    service(PayrollService, { listPayslips: () => ({ payslips: [] }) });
  });

  expect(await screen.findByText("まだ確定した給与明細はありません。")).toBeInTheDocument();
});

test("明細行には作った時点の案件名を出す", async () => {
  renderWithApi(<MyPayslipsPage staffId={1n} />, ({ service }) => {
    service(PayrollService, {
      listPayslips: () => ({
        payslips: [
          create(PayslipSchema, {
            payslipId: 5n,
            staffId: 1n,
            payYear: 2026,
            payMonth: 8,
            status: PayslipStatus.FINALIZED,
            totalYen: 9_300n,
            lines: [
              { projectId: 10n, projectName: "新宿の倉庫", workMinutes: 465, hourlyRate: 1_200n, amountYen: 9_300n },
            ],
          }),
        ],
      }),
    });
  });

  const payslip = await screen.findByRole("region", { name: "2026年8月分の給与明細" });
  expect(payslip).toHaveTextContent("確定済み");
  expect(screen.getByRole("cell", { name: "新宿の倉庫" })).toBeInTheDocument();
  expect(screen.getByRole("cell", { name: "7時間45分" })).toBeInTheDocument();
});
