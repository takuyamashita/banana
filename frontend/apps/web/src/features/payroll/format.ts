import { PayslipStatus } from "@platform/api-client";

export const yen = new Intl.NumberFormat("ja-JP", { style: "currency", currency: "JPY" });

/// 稼働時間を「7時間45分」の形にする(ちょうどの時間なら「8時間」)
export function formatWorkMinutes(minutes: number): string {
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (rest === 0) return `${hours}時間`;
  return hours === 0 ? `${rest}分` : `${hours}時間${rest}分`;
}

export function statusLabel(status: PayslipStatus): string {
  switch (status) {
    case PayslipStatus.DRAFT:
      return "作成中";
    case PayslipStatus.FINALIZED:
      return "確定済み";
    case PayslipStatus.UNSPECIFIED:
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

/// 給与明細を作る月の既定値。月初に前の月の分を作るので、前の月にする
export function previousMonth(today: Date): { year: number; month: number } {
  const month = today.getMonth(); // 0 始まりなので、そのまま前の月の番号になる
  return month === 0 ? { year: today.getFullYear() - 1, month: 12 } : { year: today.getFullYear(), month };
}
