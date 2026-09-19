import type { Payslip } from "@platform/api-client";
import { numberClass } from "@platform/ui";

import { formatWorkMinutes, statusLabel, yen } from "./format";

/// 給与明細1件の中身。案件名は明細を作った時点のもの
export function PayslipView({ payslip }: { payslip: Payslip }) {
  const title = `${payslip.payYear}年${payslip.payMonth}月分の給与明細`;
  return (
    <section aria-label={title}>
      <p>
        {payslip.payYear}年{payslip.payMonth}月分 ・ {statusLabel(payslip.status)} ・ 合計{" "}
        <strong data-testid="payslip-total">{yen.format(payslip.totalYen)}</strong>
      </p>
      <table>
        <thead>
          <tr>
            <th scope="col">案件</th>
            <th scope="col" className={numberClass()}>
              稼働
            </th>
            <th scope="col" className={numberClass()}>
              時給
            </th>
            <th scope="col" className={numberClass()}>
              金額
            </th>
          </tr>
        </thead>
        <tbody>
          {payslip.lines.map((line, i) => (
            // 明細行に番号はなく、並びは作ったときのまま変わらないので、位置で見分ける
            <tr key={i}>
              <td>{line.projectName}</td>
              <td className={numberClass()}>{formatWorkMinutes(line.workMinutes)}</td>
              <td className={numberClass()}>{yen.format(line.hourlyRate)}</td>
              <td className={numberClass()}>{yen.format(line.amountYen)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
