import type { MessageInitShape } from "@bufbuild/protobuf";
import type { CreatePayslipRequestSchema, PayslipLineInputSchema } from "@platform/api-client";

export interface LineInput {
  /// 画面の中だけで行を見分ける番号(行を消しても、他の行の入力が入れ替わらないように)
  key: number;
  projectId: string;
  workMinutes: string;
  hourlyRate: string;
}

let nextLineKey = 0;
export const newLine = (): LineInput => ({ key: nextLineKey++, projectId: "", workMinutes: "", hourlyRate: "" });

type CreateInit = MessageInitShape<typeof CreatePayslipRequestSchema>;
type LineInit = MessageInitShape<typeof PayslipLineInputSchema>;

/// 入力を API に送る形にする。ここで見るのは数字として読めるかだけで、
/// 稼働は15分単位か・時給の上限などの業務の決まりはサーバー(domain)が確かめて文言を返す
export function toCreateRequest(
  staffId: string,
  year: string,
  month: string,
  lines: LineInput[],
): { request: CreateInit } | { error: string } {
  if (!staffId) return { error: "派遣社員を選んでください。" };
  const payYear = toInteger(year);
  const payMonth = toInteger(month);
  if (payYear === null || payMonth === null) return { error: "年と月は数字で入力してください。" };

  const parsed: LineInit[] = [];
  for (const [i, line] of lines.entries()) {
    const label = `明細 ${i + 1}`;
    if (!line.projectId) return { error: `${label}の案件を選んでください。` };
    const workMinutes = toInteger(line.workMinutes);
    if (workMinutes === null) return { error: `${label}の稼働(分)は数字で入力してください。` };
    const hourlyRate = toInteger(line.hourlyRate);
    if (hourlyRate === null) return { error: `${label}の時給(円)は数字で入力してください。` };
    parsed.push({ projectId: BigInt(line.projectId), workMinutes, hourlyRate: BigInt(hourlyRate) });
  }
  return { request: { staffId: BigInt(staffId), payYear, payMonth, lines: parsed } };
}

/// 0 以上の整数として読めれば、その数。全角数字は半角にしてから読む
function toInteger(value: string): number | null {
  const normalized = value.trim().replace(/[０-９]/g, (c) => String.fromCharCode(c.charCodeAt(0) - 0xfee0));
  if (!/^\d+$/.test(normalized)) return null;
  const n = Number(normalized);
  return Number.isSafeInteger(n) ? n : null;
}
