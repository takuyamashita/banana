import { monthRange, monthSearch, shiftMonth, thisMonth } from "./month";

test("URL の引数の年月は、要求のスキーマで読めて範囲の中のものだけ受け付ける", () => {
  expect(monthSearch({ year: "2026", month: "9" })).toEqual({ year: 2026, month: 9 });
  expect(monthSearch({ year: "2026", month: "13" })).toEqual({});
  expect(monthSearch({ year: "abc", month: "9" })).toEqual({});
  expect(monthSearch({ year: "2026" })).toEqual({});
});

test("月をずらすと年をまたぐ", () => {
  expect(shiftMonth({ year: 2026, month: 12 }, 1)).toEqual({ year: 2027, month: 1 });
  expect(shiftMonth({ year: 2026, month: 1 }, -1)).toEqual({ year: 2025, month: 12 });
  expect(thisMonth(new Date(2026, 8, 20))).toEqual({ year: 2026, month: 9 });
});

test("日付の入力欄の範囲は、その月の初日から末日まで(うるう年も)", () => {
  expect(monthRange({ year: 2028, month: 2 })).toEqual({ first: "2028-02-01", last: "2028-02-29" });
  expect(monthRange({ year: 2026, month: 9 })).toEqual({ first: "2026-09-01", last: "2026-09-30" });
});
