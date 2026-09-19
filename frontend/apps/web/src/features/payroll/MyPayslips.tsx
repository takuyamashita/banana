import { useQuery } from "@connectrpc/connect-query";
import { PayrollService } from "@platform/api-client";
import { Alert, Card } from "@platform/ui";

import { errorMessage } from "../../lib/errors";
import { PayslipView } from "./PayslipView";

/// 派遣社員本人が自分の給与明細を見る画面。本人には確定済みの給与明細だけが届く
export function MyPayslips({ staffId }: { staffId: bigint }) {
  const { data, error, isPending } = useQuery(PayrollService.method.listPayslips, { staffId });

  return (
    <Card title="自分の給与明細">
      {isPending && <output>読み込み中…</output>}
      {error && <Alert>{errorMessage(error)}</Alert>}
      {data?.payslips.length === 0 && <p>まだ確定した給与明細はありません。</p>}
      {data?.payslips.map((p) => (
        <PayslipView key={String(p.payslipId)} payslip={p} />
      ))}
    </Card>
  );
}
