import { Alert, Button, Card, clusterClass, Field } from "@platform/ui";

import { formatMinutes, monthLabel } from "./month";
import { EntriesTable } from "./MyTimesheet";
import { useTimesheetApproval, type TimesheetApprovalModel } from "./useTimesheetApproval";

/// 管理者が、申告された勤務表を確かめて承認するか、理由を付けて差し戻す画面
export function TimesheetApprovalPage() {
  return <TimesheetApprovalView {...useTimesheetApproval()} />;
}

export function TimesheetApprovalView(model: TimesheetApprovalModel) {
  return (
    <Card title="勤怠の承認">
      {model.error && <Alert>{model.error}</Alert>}
      {model.notice && <Alert tone="success">{model.notice}</Alert>}
      {model.submitted.length === 0 && <p>承認を待っている勤務表はありません。</p>}
      {model.submitted.map((sheet) => {
        const title = `${sheet.staffName}さんの${monthLabel(sheet)}の勤務表`;
        return (
          <section key={String(sheet.timesheetId)} aria-label={title}>
            <h3>{title}</h3>
            <p>
              合計 <strong>{formatMinutes(sheet.totalMinutes)}</strong>
            </p>
            <EntriesTable entries={sheet.entries} />
            <Field
              label="差し戻すときの理由"
              value={model.reasonOf(sheet.timesheetId)}
              onChange={(e) => model.onReasonChange(sheet.timesheetId, e.target.value)}
            />
            <div className={clusterClass()}>
              <Button disabled={model.busy} onClick={() => model.onApprove(sheet.timesheetId)}>
                承認する
              </Button>
              <Button variant="secondary" disabled={model.busy} onClick={() => model.onReturn(sheet.timesheetId)}>
                差し戻す
              </Button>
            </div>
          </section>
        );
      })}
    </Card>
  );
}
