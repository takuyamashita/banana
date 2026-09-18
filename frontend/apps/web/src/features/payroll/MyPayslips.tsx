import type { Payslip } from "@platform/api-client";
import { Alert, Card } from "@platform/ui";
import { useEffect, useState } from "react";

import { errorMessage, useApi } from "../../lib/api";
import { PayslipView } from "./PayslipView";

/// 派遣社員本人が自分の給与明細を見る画面
export function MyPayslips({ staffId }: { staffId: bigint }) {
  const api = useApi();
  const [payslips, setPayslips] = useState<Payslip[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.payroll
      .listPayslips({ staffId })
      .then((res) => setPayslips(res.payslips))
      .catch((err: unknown) => setError(errorMessage(err)));
  }, [api, staffId]);

  return (
    <Card title="自分の給与明細">
      {error && <Alert>{error}</Alert>}
      {payslips?.length === 0 && <p>まだ確定した給与明細はありません。</p>}
      {payslips?.map((p) => (
        <PayslipView key={String(p.payslipId)} payslip={p} />
      ))}
    </Card>
  );
}
