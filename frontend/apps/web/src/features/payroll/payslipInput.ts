import { fromJson } from "@bufbuild/protobuf";
import {
  CreatePayslipRequestSchema,
  PayslipLineInputSchema,
  type CreatePayslipRequest,
  type CreatePayslipRequestJson,
  type PayslipLineInputJson,
} from "@platform/api-client";

import { checkField as check, normalize, type Fields } from "../../lib/form";

type LineField = keyof Fields<PayslipLineInputJson>;
type PeriodField = "payYear" | "payMonth";

/// 明細行の入力。key は画面の中だけで行を見分ける番号(行を消しても、他の行の入力が入れ替わらない)
export type LineDraft = Fields<PayslipLineInputJson> & { key: number };

/// 給与明細の作成の入力。派遣社員は URL で選ぶので含めない
export type PayslipDraft = Pick<Fields<CreatePayslipRequestJson>, PeriodField> & {
  lines: LineDraft[];
  /// 次に足す行の key
  nextKey: number;
};

export type DraftAction =
  | { type: "setPeriod"; field: PeriodField; value: string }
  | { type: "setLine"; key: number; field: LineField; value: string }
  | { type: "addLine" }
  | { type: "removeLine"; key: number }
  | { type: "clearLines" }
  /// 勤怠で承認された案件ごとの稼働で明細行を作り直す。同じ案件の行に入れてあった時給は残す
  | { type: "fillFromApprovedWork"; work: readonly { projectId: string; workMinutes: string }[] };

const emptyLine = (key: number): LineDraft => ({ key, projectId: "", workMinutes: "", hourlyRate: "" });

export function initialDraft(period: { year: number; month: number }): PayslipDraft {
  return { payYear: String(period.year), payMonth: String(period.month), lines: [emptyLine(0)], nextKey: 1 };
}

export function draftReducer(draft: PayslipDraft, action: DraftAction): PayslipDraft {
  switch (action.type) {
    case "setPeriod":
      return { ...draft, [action.field]: action.value };
    case "setLine":
      return {
        ...draft,
        lines: draft.lines.map((line) => (line.key === action.key ? { ...line, [action.field]: action.value } : line)),
      };
    case "addLine":
      return { ...draft, lines: [...draft.lines, emptyLine(draft.nextKey)], nextKey: draft.nextKey + 1 };
    case "removeLine":
      return { ...draft, lines: draft.lines.filter((line) => line.key !== action.key) };
    case "clearLines":
      return { ...draft, lines: [emptyLine(draft.nextKey)], nextKey: draft.nextKey + 1 };
    case "fillFromApprovedWork": {
      const lines = action.work.map((w, i) => ({
        key: draft.nextKey + i,
        projectId: w.projectId,
        workMinutes: w.workMinutes,
        hourlyRate: draft.lines.find((line) => line.projectId === w.projectId)?.hourlyRate ?? "",
      }));
      return { ...draft, lines, nextKey: draft.nextKey + lines.length };
    }
    default: {
      // 操作を足したら、ここで型エラーになる
      const unknown: never = action;
      return unknown;
    }
  }
}

/// 画面の見出し。エラーの文言にも使う
export const labels = {
  payYear: "年",
  payMonth: "月",
  projectId: "案件",
  workMinutes: "稼働(分)",
  hourlyRate: "時給(円)",
} as const satisfies Record<PeriodField | LineField, string>;

/// 入力を API の要求にする。ここで見るのは「入力があるか」と「要求の型として読めるか」だけで、読めるかは
/// 生成したスキーマ(fromJson)で決める。稼働は15分単位か・時給の上限などの業務の決まりはサーバーが確かめる
export function toCreateRequest(
  staffId: string | undefined,
  draft: PayslipDraft,
): { request: CreatePayslipRequest } | { error: string } {
  if (!staffId) return { error: "派遣社員を選んでください。" };
  for (const field of ["payYear", "payMonth"] as const) {
    const error = check(CreatePayslipRequestSchema, field, draft[field], labels[field]);
    if (error) return { error };
  }
  for (const [i, line] of draft.lines.entries()) {
    if (!line.projectId) return { error: `明細 ${i + 1}の${labels.projectId}を選んでください。` };
    for (const field of ["workMinutes", "hourlyRate"] as const) {
      const error = check(PayslipLineInputSchema, field, line[field], `明細 ${i + 1}の${labels[field]}`);
      if (error) return { error };
    }
  }
  const json: CreatePayslipRequestJson = {
    staffId,
    payYear: Number(normalize(draft.payYear)),
    payMonth: Number(normalize(draft.payMonth)),
    lines: draft.lines.map((line) => ({
      projectId: line.projectId,
      workMinutes: Number(normalize(line.workMinutes)),
      hourlyRate: normalize(line.hourlyRate),
    })),
  };
  return { request: fromJson(CreatePayslipRequestSchema, json) };
}

/// 入力中の対象月。年と月が数字として読めなければ undefined(承認された稼働を取りに行かない)
export function draftPeriod(draft: PayslipDraft): { payYear: number; payMonth: number } | undefined {
  const payYear = Number(normalize(draft.payYear));
  const payMonth = Number(normalize(draft.payMonth));
  if (!Number.isInteger(payYear) || !Number.isInteger(payMonth)) return undefined;
  if (payYear < 2000 || payYear > 2999 || payMonth < 1 || payMonth > 12) return undefined;
  return { payYear, payMonth };
}
