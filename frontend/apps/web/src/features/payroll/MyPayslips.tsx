import { useSuspenseQuery } from "@connectrpc/connect-query";
import { PayrollService, type Payslip } from "@platform/api-client";
import { Alert, Card } from "@platform/ui";

import { PayslipView } from "./PayslipView";

/// 派遣社員本人が自分の給与明細を見る画面。本人には確定済みの給与明細だけが届く
export function MyPayslipsPage({ staffId }: { staffId: bigint }) {
  const { payslips } = useSuspenseQuery(PayrollService.method.listPayslips, { staffId }).data;
  return <MyPayslipsView payslips={payslips} />;
}

export function MyPayslipsView({ payslips }: { payslips: Payslip[] }) {
  return (
    <Card title="自分の給与明細">
      {payslips.length === 0 && <p>まだ確定した給与明細はありません。</p>}
      {payslips.map((p) => (
        <PayslipView key={String(p.payslipId)} payslip={p} />
      ))}
    </Card>
  );
}

/// ログインはできたが、派遣社員として登録されていない
export function NotRegisteredAsStaff() {
  return <Alert>派遣社員として登録されていません。管理者に連絡してください。</Alert>;
}
