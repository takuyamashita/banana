import { PayslipStatus, type Payslip } from "@platform/api-client";

const yen = new Intl.NumberFormat("ja-JP", { style: "currency", currency: "JPY" });

export function PayslipView({ payslip, projectName }: { payslip: Payslip; projectName?: (id: bigint) => string }) {
  return (
    <div className="payslip" aria-label={`${payslip.payYear}年${payslip.payMonth}月分の給与明細`}>
      <p>
        {payslip.payYear}年{payslip.payMonth}月分 ・{" "}
        {payslip.status === PayslipStatus.FINALIZED ? "確定済み" : "下書き"} ・ 合計{" "}
        <strong data-testid="payslip-total">{yen.format(payslip.totalYen)}</strong>
      </p>
      <table>
        <thead>
          <tr>
            <th>案件</th>
            <th className="num">稼働(分)</th>
            <th className="num">時給</th>
            <th className="num">金額</th>
          </tr>
        </thead>
        <tbody>
          {payslip.lines.map((line, i) => (
            <tr key={i}>
              <td>{projectName ? projectName(line.projectId) : `#${line.projectId}`}</td>
              <td className="num">{line.workMinutes}</td>
              <td className="num">{yen.format(line.hourlyRate)}</td>
              <td className="num">{yen.format(line.amountYen)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
