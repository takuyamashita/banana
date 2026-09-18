import type { Payslip, Project, Staff } from "@platform/api-client";
import { Alert, Button, Card, Field } from "@platform/ui";
import { useEffect, useState, type FormEvent } from "react";

import { errorMessage, useApi } from "../../lib/api";
import { PayslipView } from "./PayslipView";

interface LineInput {
  projectId: string;
  workMinutes: string;
  hourlyRate: string;
}

const emptyLine: LineInput = { projectId: "", workMinutes: "", hourlyRate: "" };

/// 管理者が派遣社員の月次給与を確定する画面
export function FinalizePayslipForm() {
  const api = useApi();
  const [staff, setStaff] = useState<Staff[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [staffId, setStaffId] = useState("");
  const [year, setYear] = useState(String(new Date().getFullYear()));
  const [month, setMonth] = useState(String(new Date().getMonth() + 1));
  const [lines, setLines] = useState<LineInput[]>([{ ...emptyLine }]);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<Payslip | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    Promise.all([api.staff.listStaff({}), api.project.listProjects({})])
      .then(([s, p]) => {
        setStaff(s.staff);
        setProjects(p.projects);
      })
      .catch((err: unknown) => setError(errorMessage(err)));
  }, [api]);

  const updateLine = (index: number, patch: Partial<LineInput>) =>
    setLines((prev) => prev.map((line, i) => (i === index ? { ...line, ...patch } : line)));

  const projectName = (id: bigint) => projects.find((p) => p.projectId === id)?.name ?? `#${id}`;

  async function submit(event: FormEvent) {
    event.preventDefault();
    setError(null);
    setResult(null);
    setSubmitting(true);
    try {
      // 入力の検証はサーバー(domain)が行う。ここでは型の変換だけ
      const { payslipId } = await api.payroll.finalizePayslip({
        staffId: BigInt(staffId || 0),
        payYear: Number(year),
        payMonth: Number(month),
        lines: lines.map((l) => ({
          projectId: BigInt(l.projectId || 0),
          workMinutes: Number(l.workMinutes),
          hourlyRate: BigInt(l.hourlyRate || 0),
        })),
      });
      const { payslip } = await api.payroll.getPayslip({ payslipId });
      setResult(payslip ?? null);
      setLines([{ ...emptyLine }]);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Card title="給与確定">
      <form onSubmit={(e) => void submit(e)}>
        <div className="ui-field">
          <label htmlFor="finalize-staff">派遣社員</label>
          <select id="finalize-staff" value={staffId} onChange={(e) => setStaffId(e.target.value)} required>
            <option value="">選択してください</option>
            {staff.map((s) => (
              <option key={String(s.staffId)} value={String(s.staffId)}>
                {s.displayName}({s.email})
              </option>
            ))}
          </select>
        </div>
        <div className="row">
          <Field label="年" type="number" value={year} onChange={(e) => setYear(e.target.value)} required />
          <Field label="月" type="number" value={month} onChange={(e) => setMonth(e.target.value)} required />
        </div>

        {lines.map((line, i) => (
          <fieldset key={i} className="line" aria-label={`明細 ${i + 1}`}>
            <legend>明細 {i + 1}</legend>
            <div className="ui-field">
              <label htmlFor={`line-project-${i}`}>案件</label>
              <select
                id={`line-project-${i}`}
                value={line.projectId}
                onChange={(e) => updateLine(i, { projectId: e.target.value })}
                required
              >
                <option value="">選択してください</option>
                {projects.map((p) => (
                  <option key={String(p.projectId)} value={String(p.projectId)}>
                    {p.name}
                  </option>
                ))}
              </select>
            </div>
            <div className="row">
              <Field
                label="稼働(分)"
                type="number"
                value={line.workMinutes}
                onChange={(e) => updateLine(i, { workMinutes: e.target.value })}
                required
              />
              <Field
                label="時給(円)"
                type="number"
                value={line.hourlyRate}
                onChange={(e) => updateLine(i, { hourlyRate: e.target.value })}
                required
              />
            </div>
          </fieldset>
        ))}

        <div className="actions">
          <Button variant="secondary" onClick={() => setLines((prev) => [...prev, { ...emptyLine }])}>
            明細を追加
          </Button>
          <Button type="submit" disabled={submitting}>
            確定する
          </Button>
        </div>
      </form>

      {error && <Alert>{error}</Alert>}
      {result && (
        <>
          <Alert tone="success">給与明細 #{String(result.payslipId)} を確定しました。</Alert>
          <PayslipView payslip={result} projectName={projectName} />
        </>
      )}
    </Card>
  );
}
