import { createConnectQueryKey, skipToken, useMutation, useQuery } from "@connectrpc/connect-query";
import { PayrollService, PayslipStatus, ProjectService, StaffService, type Payslip } from "@platform/api-client";
import { Alert, Button, Card, Field } from "@platform/ui";
import { useQueryClient } from "@tanstack/react-query";
import { useState, type FormEvent } from "react";

import { errorMessage } from "../../lib/errors";
import { previousMonth } from "./format";
import { newLine, toCreateRequest, type LineInput } from "./payslipInput";
import { PayslipView } from "./PayslipView";

/// 管理者が派遣社員の月次給与明細を作成し、内容を確かめてから確定する画面
export function PayslipAdmin() {
  const queryClient = useQueryClient();
  const staff = useQuery(StaffService.method.listStaff, {});
  const projects = useQuery(ProjectService.method.listProjects, {});

  const initial = previousMonth(new Date());
  const [staffId, setStaffId] = useState("");
  const [year, setYear] = useState(String(initial.year));
  const [month, setMonth] = useState(String(initial.month));
  const [lines, setLines] = useState<LineInput[]>(() => [newLine()]);
  const [inputError, setInputError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  // 一覧は派遣社員ごとに別に持つ。選び直した直後は、新しい人の一覧が届くまで何も出さない
  // (前の人の給与明細と確定ボタンを出したままにしない)
  const payslips = useQuery(PayrollService.method.listPayslips, staffId ? { staffId: BigInt(staffId) } : skipToken);
  const refreshPayslips = () =>
    queryClient.invalidateQueries({
      queryKey: createConnectQueryKey({ schema: PayrollService.method.listPayslips, cardinality: "finite" }),
    });

  const createPayslip = useMutation(PayrollService.method.createPayslip, {
    onSuccess: async ({ payslipId }) => {
      setLines([newLine()]);
      setNotice(`給与明細 #${payslipId} を作成しました。内容を確かめて確定してください。`);
      await refreshPayslips();
    },
  });
  const finalizePayslip = useMutation(PayrollService.method.finalizePayslip, {
    onSuccess: async (_, { payslipId }) => {
      setNotice(`給与明細 #${payslipId} を確定しました。`);
      await refreshPayslips();
    },
  });
  const busy = createPayslip.isPending || finalizePayslip.isPending;

  const staffName = staff.data?.staff.find((s) => String(s.staffId) === staffId)?.displayName ?? "";
  const updateLine = (key: number, patch: Partial<LineInput>) =>
    setLines((prev) => prev.map((line) => (line.key === key ? { ...line, ...patch } : line)));

  function clearMessages() {
    setInputError(null);
    setNotice(null);
    createPayslip.reset();
    finalizePayslip.reset();
  }

  function create(event: FormEvent) {
    event.preventDefault();
    clearMessages();
    const result = toCreateRequest(staffId, year, month, lines);
    if ("error" in result) {
      setInputError(result.error);
      return;
    }
    createPayslip.mutate(result.request);
  }

  function finalize(payslip: Payslip) {
    clearMessages();
    finalizePayslip.mutate({ payslipId: payslip.payslipId });
  }

  const error =
    inputError ??
    [staff.error, projects.error, payslips.error, createPayslip.error, finalizePayslip.error]
      .filter((e) => e !== null)
      .map(errorMessage)[0];

  return (
    <Card title="給与明細">
      {/* 入力の決まりはサーバーが確かめて日本語で返すので、ブラウザの検証(英語のこともある)は使わない */}
      <form onSubmit={create} aria-label="給与明細の作成" noValidate>
        <div className="ui-field">
          <label htmlFor="payslip-staff">派遣社員</label>
          <select id="payslip-staff" value={staffId} onChange={(e) => setStaffId(e.target.value)} required>
            <option value="">選択してください</option>
            {staff.data?.staff.map((s) => (
              <option key={String(s.staffId)} value={String(s.staffId)}>
                {s.displayName}({s.email})
              </option>
            ))}
          </select>
        </div>

        <div className="row">
          <Field label="年" inputMode="numeric" value={year} onChange={(e) => setYear(e.target.value)} required />
          <Field label="月" inputMode="numeric" value={month} onChange={(e) => setMonth(e.target.value)} required />
        </div>

        {lines.map((line, i) => (
          <fieldset key={line.key} className="line">
            <legend>明細 {i + 1}</legend>
            <div className="ui-field">
              <label htmlFor={`line-project-${line.key}`}>案件</label>
              <select
                id={`line-project-${line.key}`}
                value={line.projectId}
                onChange={(e) => updateLine(line.key, { projectId: e.target.value })}
                required
              >
                <option value="">選択してください</option>
                {projects.data?.projects.map((p) => (
                  <option key={String(p.projectId)} value={String(p.projectId)}>
                    {p.name}
                  </option>
                ))}
              </select>
            </div>
            <div className="row">
              <Field
                label="稼働(分)"
                inputMode="numeric"
                placeholder="15分単位"
                value={line.workMinutes}
                onChange={(e) => updateLine(line.key, { workMinutes: e.target.value })}
                required
              />
              <Field
                label="時給(円)"
                inputMode="numeric"
                value={line.hourlyRate}
                onChange={(e) => updateLine(line.key, { hourlyRate: e.target.value })}
                required
              />
            </div>
            {lines.length > 1 && (
              <Button
                variant="secondary"
                onClick={() => setLines((prev) => prev.filter((l) => l.key !== line.key))}
                aria-label={`明細 ${i + 1} を削除`}
              >
                削除
              </Button>
            )}
          </fieldset>
        ))}

        <div className="actions">
          <Button variant="secondary" onClick={() => setLines((prev) => [...prev, newLine()])}>
            明細を追加
          </Button>
          <Button type="submit" disabled={busy}>
            作成する
          </Button>
        </div>
      </form>

      {error && <Alert>{error}</Alert>}
      {notice && <Alert tone="success">{notice}</Alert>}

      {payslips.isLoading && <output>読み込み中…</output>}
      {payslips.data?.payslips.map((p) => (
        <section key={String(p.payslipId)} className="payslip-item">
          <PayslipView payslip={p} />
          {p.status === PayslipStatus.DRAFT && (
            <Button disabled={busy} onClick={() => finalize(p)}>
              {`${staffName}の${p.payYear}年${p.payMonth}月分を確定する`}
            </Button>
          )}
        </section>
      ))}
    </Card>
  );
}
