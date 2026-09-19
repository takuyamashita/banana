import { create, fromJson } from "@bufbuild/protobuf";
import { timesheet } from "@platform/api-client";

import { checkField, normalize, type Fields } from "../../lib/form";
import type { TimesheetMonth } from "./month";

type EntryField = keyof Fields<timesheet.WorkEntryInputJson>;

/// 稼働の1行の入力。key は画面の中だけで行を見分ける番号(行を消しても、他の行の入力が入れ替わらない)
export type EntryDraft = Fields<timesheet.WorkEntryInputJson> & { key: number };

/// 勤務表の入力
export interface TimesheetDraft {
  entries: EntryDraft[];
  /// 次に足す行の key
  nextKey: number;
}

export type DraftAction =
  | { type: "setEntry"; key: number; field: EntryField; value: string }
  | { type: "addEntry"; date: string }
  | { type: "removeEntry"; key: number };

/// サーバーから届いた勤務表を、入力の形にする
export function initialDraft(sheet: timesheet.Timesheet): TimesheetDraft {
  const entries = sheet.entries.map((e, key) => ({
    key,
    date: e.date,
    projectId: String(e.projectId),
    workMinutes: String(e.workMinutes),
  }));
  return { entries, nextKey: entries.length };
}

export function draftReducer(draft: TimesheetDraft, action: DraftAction): TimesheetDraft {
  switch (action.type) {
    case "setEntry":
      return {
        ...draft,
        entries: draft.entries.map((e) => (e.key === action.key ? { ...e, [action.field]: action.value } : e)),
      };
    case "addEntry": {
      // 直前の行と同じ案件で書き始めることが多いので、案件は引き継ぐ
      const previous = draft.entries.at(-1);
      const entry = { key: draft.nextKey, date: action.date, projectId: previous?.projectId ?? "", workMinutes: "" };
      return { entries: [...draft.entries, entry], nextKey: draft.nextKey + 1 };
    }
    case "removeEntry":
      return { ...draft, entries: draft.entries.filter((e) => e.key !== action.key) };
    default: {
      // 操作を足したら、ここで型エラーになる
      const unknown: never = action;
      return unknown;
    }
  }
}

/// 画面の見出し。エラーの文言にも使う
export const labels = {
  date: "日付",
  projectId: "案件",
  workMinutes: "稼働(分)",
} as const satisfies Record<EntryField, string>;

/// 入力を API の要求にする。ここで見るのは「入力があるか」と「要求の型として読めるか」だけ。
/// 15分単位・対象月の日付・1日24時間までといった決まりはサーバーが確かめる
export function toSaveRequest(
  month: TimesheetMonth,
  draft: TimesheetDraft,
): { request: timesheet.SaveMyTimesheetRequest } | { error: string } {
  const entries: timesheet.WorkEntryInputJson[] = [];
  for (const [i, entry] of draft.entries.entries()) {
    const row = `${i + 1}行目の`;
    if (!entry.date) return { error: `${row}${labels.date}を入力してください。` };
    if (!entry.projectId) return { error: `${row}${labels.projectId}を選んでください。` };
    const error = checkField(
      timesheet.WorkEntryInputSchema,
      "workMinutes",
      entry.workMinutes,
      `${row}${labels.workMinutes}`,
    );
    if (error) return { error };
    entries.push({ date: entry.date, projectId: entry.projectId, workMinutes: Number(normalize(entry.workMinutes)) });
  }
  return {
    request: create(timesheet.SaveMyTimesheetRequestSchema, {
      year: month.year,
      month: month.month,
      entries: entries.map((e) => fromJson(timesheet.WorkEntryInputSchema, e)),
    }),
  };
}

/// 入力中の稼働の合計(分)。まだ数字として読めない行は数えない
export function draftTotalMinutes(draft: TimesheetDraft): number {
  return draft.entries.reduce((sum, e) => {
    const minutes = Number(normalize(e.workMinutes));
    return Number.isInteger(minutes) && minutes > 0 ? sum + minutes : sum;
  }, 0);
}

/// 行を足すときの日付。最後の行の次の日(月末を越えない)、行がなければ月の初日
export function nextEntryDate(draft: TimesheetDraft, range: { first: string; last: string }): string {
  const last = draft.entries.at(-1)?.date;
  if (!last) return range.first;
  const next = new Date(`${last}T00:00:00Z`);
  next.setUTCDate(next.getUTCDate() + 1);
  const date = next.toISOString().slice(0, 10);
  return date > range.last ? range.last : date;
}
