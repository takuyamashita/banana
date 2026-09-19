import { draftReducer, draftTotalMinutes, nextEntryDate, toSaveRequest, type TimesheetDraft } from "./timesheetInput";

const september = { year: 2026, month: 9 };
const range = { first: "2026-09-01", last: "2026-09-30" };

function draftWith(...rows: [string, string, string][]): TimesheetDraft {
  return {
    entries: rows.map(([date, projectId, workMinutes], key) => ({ key, date, projectId, workMinutes })),
    nextKey: rows.length,
  };
}

test("足した行は直前の行の案件を引き継ぎ、日付は次の日(月末を越えない)", () => {
  const draft = draftReducer(draftWith(["2026-09-01", "10", "480"]), {
    type: "addEntry",
    date: nextEntryDate(draftWith(["2026-09-01", "10", "480"]), range),
  });
  expect(draft.entries[1]).toEqual({ key: 1, date: "2026-09-02", projectId: "10", workMinutes: "" });
  expect(nextEntryDate(draftWith(["2026-09-30", "10", "480"]), range)).toBe("2026-09-30");
  expect(nextEntryDate(draftWith(), range)).toBe("2026-09-01");
});

test("入力が足りない行・数字でない稼働は、送らずにどこが違うかを伝える", () => {
  expect(toSaveRequest(september, draftWith(["", "10", "480"]))).toEqual({ error: "1行目の日付を入力してください。" });
  expect(toSaveRequest(september, draftWith(["2026-09-01", "", "480"]))).toEqual({
    error: "1行目の案件を選んでください。",
  });
  expect(toSaveRequest(september, draftWith(["2026-09-01", "10", "8時間"]))).toEqual({
    error: "1行目の稼働(分)は数字で入力してください。",
  });
});

test("読める入力は要求の形にする(全角の数字も読む。15分単位かはサーバーが確かめる)", () => {
  const result = toSaveRequest(september, draftWith(["2026-09-01", "10", "４７０"]));
  if (!("request" in result)) throw new Error(result.error);
  expect(result.request.year).toBe(2026);
  expect(result.request.entries[0]).toMatchObject({ date: "2026-09-01", projectId: 10n, workMinutes: 470 });
});

test("合計は読める行だけを数える", () => {
  expect(draftTotalMinutes(draftWith(["2026-09-01", "10", "480"], ["2026-09-02", "10", ""]))).toBe(480);
});
