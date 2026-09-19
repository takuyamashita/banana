import { fromJson, toJson } from "@bufbuild/protobuf";
import { timesheet } from "@platform/api-client";

/// 勤怠の画面の URL の引数(対象月)。勤務表の取得の要求(GetMyTimesheetRequest)の JSON 形の年と月
export type TimesheetMonth = Required<Pick<timesheet.GetMyTimesheetRequestJson, "year" | "month">>;

/// URL の引数を、勤務表の取得の要求のスキーマで読む。読めない・範囲の外なら指定がないことにする
export function monthSearch(raw: Record<string, unknown>): Partial<TimesheetMonth> {
  const { year, month } = raw;
  if (typeof year !== "string" || typeof month !== "string") return {};
  try {
    const json = toJson(
      timesheet.GetMyTimesheetRequestSchema,
      fromJson(timesheet.GetMyTimesheetRequestSchema, { year, month }),
    );
    if (json.year === undefined || json.month === undefined) return {};
    if (json.year < 2000 || json.year > 2999 || json.month < 1 || json.month > 12) return {};
    return { year: json.year, month: json.month };
  } catch {
    return {};
  }
}

/// 今日を含む月(勤怠は、その月のうちに毎日書く)
export function thisMonth(today: Date): TimesheetMonth {
  return { year: today.getFullYear(), month: today.getMonth() + 1 };
}

/// `delta` か月ずらした月
export function shiftMonth({ year, month }: TimesheetMonth, delta: number): TimesheetMonth {
  const index = year * 12 + (month - 1) + delta;
  return { year: Math.floor(index / 12), month: (index % 12) + 1 };
}

export function monthLabel({ year, month }: TimesheetMonth): string {
  return `${year}年${month}月`;
}

/// その月の最初と最後の日(YYYY-MM-DD)。日付の入力欄の範囲に使う
export function monthRange({ year, month }: TimesheetMonth): { first: string; last: string } {
  const last = new Date(year, month, 0).getDate();
  const mm = String(month).padStart(2, "0");
  return { first: `${year}-${mm}-01`, last: `${year}-${mm}-${String(last).padStart(2, "0")}` };
}

export function statusLabel(status: timesheet.TimesheetStatus): string {
  switch (status) {
    case timesheet.TimesheetStatus.DRAFT:
      return "作成中";
    case timesheet.TimesheetStatus.SUBMITTED:
      return "申告済み(承認待ち)";
    case timesheet.TimesheetStatus.APPROVED:
      return "承認済み";
    case timesheet.TimesheetStatus.UNSPECIFIED:
      return "不明";
    default:
      // 状態を足したら、ここで型エラーになる。古い画面に新しい状態が届いたときは「不明」と出す
      return unknownStatus(status);
  }
}

function unknownStatus(status: never): string {
  void status;
  return "不明";
}

/// 稼働時間を「7時間45分」の形にする(ちょうどの時間なら「8時間」)
export function formatMinutes(minutes: number): string {
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (rest === 0) return `${hours}時間`;
  return hours === 0 ? `${rest}分` : `${hours}時間${rest}分`;
}
