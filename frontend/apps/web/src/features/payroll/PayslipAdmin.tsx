import { PayslipStatus, type Payslip, type Project, type Staff } from "@platform/api-client";
import { Alert, Button, Card, Field } from "@platform/ui";
import { useCallback, useEffect, useState, type FormEvent } from "react";

import { errorMessage, useApi } from "../../lib/api";
import { PayslipView } from "./PayslipView";

interface LineInput {
  projectId: string;
  workMinutes: string;
  hourlyRate: string;
}

const emptyLine: LineInput = { projectId: "", workMinutes: "", hourlyRate: "" };

/// 取り出した給与明細の一覧と、それが誰のものか
interface Listed {
  staffId: string;
  payslips: Payslip[];
}

/// 管理者が派遣社員の月次給与明細を作成し、内容を確かめてから確定する画面
export function PayslipAdmin() {
  const api = useApi();
  const [staff, setStaff] = useState<Staff[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [staffId, setStaffId] = useState("");
  const [year, setYear] = useState(String(new Date().getFullYear()));
  const [month, setMonth] = useState(String(new Date().getMonth() + 1));
  const [lines, setLines] = useState<LineInput[]>([{ ...emptyLine }]);
  const [listed, setListed] = useState<Listed | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    Promise.all([api.staff.listStaff({}), api.project.listProjects({})])
      .then(([s, p]) => {
        setStaff(s.staff);
        setProjects(p.projects);
      })
      .catch((err: unknown) => setError(errorMessage(err)));
  }, [api]);

  // 一覧は誰のものかと組にして持つ。派遣社員を選び直した直後や、応答の順序が入れ替わったときに、
  // 前の派遣社員の給与明細(と確定ボタン)を出さないため
  const reload = useCallback(
    async (id: string) => {
      if (!id) return;
      const res = await api.payroll.listPayslips({ staffId: BigInt(id) });
      setListed({ staffId: id, payslips: res.payslips });
    },
    [api],
  );

  useEffect(() => {
    const controller = new AbortController();
    if (staffId) {
      api.payroll
        .listPayslips({ staffId: BigInt(staffId) }, { signal: controller.signal })
        .then((res) => setListed({ staffId, payslips: res.payslips }))
        .catch((err: unknown) => {
          if (!controller.signal.aborted) setError(errorMessage(err));
        });
    }
    return () => controller.abort();
  }, [api, staffId]);

  // 選んでいる派遣社員の一覧だけを出す。まだ届いていなければ読み込み中
  const shown = listed && listed.staffId === staffId ? listed.payslips : [];
  const loading = staffId !== "" && listed?.staffId !== staffId;
  const staffName = staff.find((s) => String(s.staffId) === staffId)?.displayName ?? "";

  const updateLine = (index: number, patch: Partial<LineInput>) =>
    setLines((prev) => prev.map((line, i) => (i === index ? { ...line, ...patch } : line)));

  const projectName = (id: bigint) => projects.find((p) => p.projectId === id)?.name ?? `#${id}`;

  async function run(action: () => Promise<string>) {
    setError(null);
    setNotice(null);
    setBusy(true);
    try {
      setNotice(await action());
      await reload(staffId);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  function create(event: FormEvent) {
    event.preventDefault();
    void run(async () => {
      // 入力の検証はサーバー(domain)が行う。ここでは型の変換だけ
      const { payslipId } = await api.payroll.createPayslip({
        staffId: BigInt(staffId || 0),
        payYear: Number(year),
        payMonth: Number(month),
        lines: lines.map((l) => ({
          projectId: BigInt(l.projectId || 0),
          workMinutes: Number(l.workMinutes),
          hourlyRate: BigInt(l.hourlyRate || 0),
        })),
      });
      setLines([{ ...emptyLine }]);
      return `給与明細 #${payslipId} を作成しました。内容を確かめて確定してください。`;
    });
  }

  function finalize(payslip: Payslip) {
    void run(async () => {
      await api.payroll.finalizePayslip({ payslipId: payslip.payslipId });
      return `給与明細 #${payslip.payslipId} を確定しました。`;
    });
  }

  return (
    <Card title="給与明細">
      <div className="ui-field">
        <label htmlFor="payslip-staff">派遣社員</label>
        <select id="payslip-staff" value={staffId} onChange={(e) => setStaffId(e.target.value)} required>
          <option value="">選択してください</option>
          {staff.map((s) => (
            <option key={String(s.staffId)} value={String(s.staffId)}>
              {s.displayName}({s.email})
            </option>
          ))}
        </select>
      </div>

      <form onSubmit={create} aria-label="給与明細の作成">
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
          <Button type="submit" disabled={busy}>
            作成する
          </Button>
        </div>
      </form>

      {error && <Alert>{error}</Alert>}
      {notice && <Alert tone="success">{notice}</Alert>}

      {loading && <output>読み込み中…</output>}
      {shown.map((p) => (
        <section key={String(p.payslipId)} className="payslip-item">
          <PayslipView payslip={p} projectName={projectName} />
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
