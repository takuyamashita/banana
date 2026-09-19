import { PayslipStatus } from "@platform/api-client";
import { Alert, Button, Card, clusterClass, cx, Field, rowClass, SelectField } from "@platform/ui";

import { formatWorkMinutes } from "./format";
import { labels } from "./payslipInput";
import { PayslipView } from "./PayslipView";
import { usePayslipAdmin, type PayslipAdminModel, type PayslipAdminProps } from "./usePayslipAdmin";

/// 管理者が派遣社員の月次給与明細を作成し、内容を確かめてから確定する画面
export function PayslipAdminPage(props: PayslipAdminProps) {
  return <PayslipAdminView {...usePayslipAdmin(props)} />;
}

export function PayslipAdminView(model: PayslipAdminModel) {
  const { draft, onEdit } = model;
  return (
    <Card title="給与明細">
      {/* 入力の決まりはサーバーが確かめて日本語で返すので、ブラウザの検証(英語のこともある)は使わない */}
      <form
        aria-label="給与明細の作成"
        noValidate
        onSubmit={(event) => {
          event.preventDefault();
          model.onCreate();
        }}
      >
        <SelectField
          label="派遣社員"
          value={model.staffId ?? ""}
          onChange={(e) => model.onSelectStaff(e.target.value || undefined)}
          required
        >
          <option value="">選択してください</option>
          {model.staff.map((s) => (
            <option key={String(s.staffId)} value={String(s.staffId)}>
              {s.displayName}({s.email})
            </option>
          ))}
        </SelectField>

        <div className={rowClass()}>
          {(["payYear", "payMonth"] as const).map((field) => (
            <Field
              key={field}
              label={labels[field]}
              inputMode="numeric"
              value={draft[field]}
              onChange={(e) => onEdit({ type: "setPeriod", field, value: e.target.value })}
              required
            />
          ))}
        </div>

        {model.approvedWork.length > 0 && (
          <div className={cx("border", "radius-1", "p-y-3", "p-x-4", "m-y-3")} aria-label="承認済みの勤怠">
            <p className={cx("m-t-0")}>
              この月の承認済みの勤怠:{" "}
              {model.approvedWork.map((w) => `${w.projectName} ${formatWorkMinutes(w.workMinutes)}`).join("、")}
            </p>
            <Button variant="secondary" onClick={model.onFillFromApprovedWork}>
              承認済みの勤怠から明細を入れる
            </Button>
          </div>
        )}

        {draft.lines.map((line, i) => (
          <fieldset key={line.key} className={cx("border", "radius-1", "m-y-3")}>
            <legend>明細 {i + 1}</legend>
            <SelectField
              label={labels.projectId}
              value={line.projectId}
              onChange={(e) => onEdit({ type: "setLine", key: line.key, field: "projectId", value: e.target.value })}
              required
            >
              <option value="">選択してください</option>
              {model.projects.map((p) => (
                <option key={String(p.projectId)} value={String(p.projectId)}>
                  {p.name}
                </option>
              ))}
            </SelectField>
            <div className={rowClass()}>
              <Field
                label={labels.workMinutes}
                inputMode="numeric"
                placeholder="15分単位"
                value={line.workMinutes}
                onChange={(e) =>
                  onEdit({ type: "setLine", key: line.key, field: "workMinutes", value: e.target.value })
                }
                required
              />
              <Field
                label={labels.hourlyRate}
                inputMode="numeric"
                value={line.hourlyRate}
                onChange={(e) => onEdit({ type: "setLine", key: line.key, field: "hourlyRate", value: e.target.value })}
                required
              />
            </div>
            {draft.lines.length > 1 && (
              <Button
                variant="secondary"
                onClick={() => onEdit({ type: "removeLine", key: line.key })}
                aria-label={`明細 ${i + 1} を削除`}
              >
                削除
              </Button>
            )}
          </fieldset>
        ))}

        <div className={clusterClass()}>
          <Button variant="secondary" onClick={() => onEdit({ type: "addLine" })}>
            明細を追加
          </Button>
          <Button type="submit" disabled={model.busy}>
            作成する
          </Button>
        </div>
      </form>

      {model.error && <Alert>{model.error}</Alert>}
      {model.notice && <Alert tone="success">{model.notice}</Alert>}

      {model.loading && <output>読み込み中…</output>}
      {model.payslips?.map((p) => (
        <section key={String(p.payslipId)}>
          <PayslipView payslip={p} />
          {p.status === PayslipStatus.DRAFT && (
            <Button disabled={model.busy} onClick={() => model.onFinalize(p)}>
              {`${model.staffName}の${p.payYear}年${p.payMonth}月分を確定する`}
            </Button>
          )}
        </section>
      ))}
    </Card>
  );
}
