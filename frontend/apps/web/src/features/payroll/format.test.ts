import { fromJson } from "@bufbuild/protobuf";
import { PayslipSchema, PayslipStatus } from "@platform/api-client";

import { formatWorkMinutes, previousMonth, statusLabel } from "./format";

test("稼働時間は時間と分で出す", () => {
  expect(formatWorkMinutes(480)).toBe("8時間");
  expect(formatWorkMinutes(465)).toBe("7時間45分");
  expect(formatWorkMinutes(45)).toBe("45分");
});

test("作る月の既定値は前の月(1月なら前の年の12月)", () => {
  expect(previousMonth(new Date(2026, 8, 19))).toEqual({ year: 2026, month: 8 });
  expect(previousMonth(new Date(2026, 0, 5))).toEqual({ year: 2025, month: 12 });
});

test("古い画面に知らない状態が届いたら「不明」と出す", () => {
  expect(statusLabel(PayslipStatus.DRAFT)).toBe("作成中");
  expect(statusLabel(PayslipStatus.FINALIZED)).toBe("確定済み");
  // 新しいサーバーが、この画面の知らない状態の番号を返した
  const fromNewerServer = fromJson(PayslipSchema, { status: 99 });
  expect(statusLabel(fromNewerServer.status)).toBe("不明");
});
