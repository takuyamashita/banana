import { Alert, Button, Card, clusterClass, cx, numberClass } from "@platform/ui";

import { formatMinutes, statusLabel } from "./month";
import { labels } from "./timesheetInput";
import { useMyTimesheet, type MyTimesheetModel, type MyTimesheetProps } from "./useMyTimesheet";

/// 派遣社員が、自分の月の勤務表に毎日の稼働を書いて申告する画面
export function MyTimesheetPage(props: MyTimesheetProps) {
  return <MyTimesheetView {...useMyTimesheet(props)} />;
}

export function MyTimesheetView(model: MyTimesheetModel) {
  return (
    <Card title={model.title}>
      <div className={clusterClass()}>
        <Button variant="secondary" onClick={model.onPreviousMonth}>
          前の月
        </Button>
        <Button variant="secondary" onClick={model.onNextMonth}>
          次の月
        </Button>
      </div>
      <p>
        {model.staffName} ・ <span data-testid="timesheet-status">{statusLabel(model.status)}</span> ・ 合計{" "}
        <strong data-testid="timesheet-total">{formatMinutes(model.totalMinutes)}</strong>
      </p>
      {model.returnedReason && <Alert>差し戻されました: {model.returnedReason}</Alert>}
      {model.editable ? <EntriesEditor {...model} /> : <EntriesTable entries={model.entries} />}
      {model.error && <Alert>{model.error}</Alert>}
      {model.notice && <Alert tone="success">{model.notice}</Alert>}
    </Card>
  );
}

/// 作成中の勤務表の入力。1日・1案件に1行
function EntriesEditor(model: MyTimesheetModel) {
  const { draft, onEdit } = model;
  return (
    // 入力の決まりはサーバーが確かめて日本語で返すので、ブラウザの検証(英語のこともある)は使わない
    <form
      aria-label="勤務表の記入"
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        model.onSave();
      }}
    >
      <table aria-label="稼働">
        <thead>
          <tr>
            <th scope="col">{labels.date}</th>
            <th scope="col">{labels.projectId}</th>
            <th scope="col">{labels.workMinutes}</th>
            <th scope="col">
              <span className={cx("fg-muted")}>操作</span>
            </th>
          </tr>
        </thead>
        <tbody>
          {draft.entries.map((entry, i) => (
            <tr key={entry.key}>
              <td>
                <input
                  type="date"
                  aria-label={`${i + 1}行目の${labels.date}`}
                  min={model.range.first}
                  max={model.range.last}
                  value={entry.date}
                  onChange={(e) => onEdit({ type: "setEntry", key: entry.key, field: "date", value: e.target.value })}
                />
              </td>
              <td>
                <select
                  aria-label={`${i + 1}行目の${labels.projectId}`}
                  value={entry.projectId}
                  onChange={(e) =>
                    onEdit({ type: "setEntry", key: entry.key, field: "projectId", value: e.target.value })
                  }
                >
                  <option value="">選択してください</option>
                  {model.projects.map((p) => (
                    <option key={String(p.projectId)} value={String(p.projectId)}>
                      {p.name}
                    </option>
                  ))}
                </select>
              </td>
              <td>
                <input
                  inputMode="numeric"
                  placeholder="15分単位"
                  aria-label={`${i + 1}行目の${labels.workMinutes}`}
                  value={entry.workMinutes}
                  onChange={(e) =>
                    onEdit({ type: "setEntry", key: entry.key, field: "workMinutes", value: e.target.value })
                  }
                />
              </td>
              <td>
                <Button
                  variant="secondary"
                  onClick={() => onEdit({ type: "removeEntry", key: entry.key })}
                  aria-label={`${i + 1}行目を削除`}
                >
                  削除
                </Button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {draft.entries.length === 0 && (
        <p className={cx("fg-muted")}>まだ稼働がありません。「行を足す」で書き始めます。</p>
      )}
      <div className={clusterClass()}>
        <Button variant="secondary" onClick={model.onAddEntry}>
          行を足す
        </Button>
        <Button type="submit" variant="secondary" disabled={model.busy}>
          保存する
        </Button>
        <Button disabled={model.busy} onClick={model.onSubmit}>
          申告する
        </Button>
      </div>
    </form>
  );
}

/// 申告・承認した勤務表の稼働(書き直せない)
export function EntriesTable({ entries }: { entries: MyTimesheetModel["entries"] }) {
  return (
    <table aria-label="稼働">
      <thead>
        <tr>
          <th scope="col">{labels.date}</th>
          <th scope="col">{labels.projectId}</th>
          <th scope="col" className={numberClass()}>
            稼働
          </th>
        </tr>
      </thead>
      <tbody>
        {entries.map((e) => (
          <tr key={`${e.date}-${String(e.projectId)}`}>
            <td>{e.date}</td>
            <td>{e.projectName}</td>
            <td className={numberClass()}>{formatMinutes(e.workMinutes)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
