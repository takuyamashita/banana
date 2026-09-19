import { fromJson, type DescMessage, type JsonObject } from "@bufbuild/protobuf";
import {
  CreatePayslipRequestSchema,
  PayslipLineInputSchema,
  type CreatePayslipRequest,
  type CreatePayslipRequestJson,
  type PayslipLineInputJson,
} from "@platform/api-client";

/// 入力欄の中身。要求の JSON 形と同じ項目を、入力されたまま(文字列で)持つ
export type Fields<T> = { [K in keyof Required<T>]: string };

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
  | { type: "clearLines" };

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

/// 1つの項目が、スキーマのその項目の型として読めるか。読めなければ文言を返す
function check(schema: DescMessage, field: string, text: string, label: string): string | undefined {
  const value = normalize(text);
  if (value === "") return `${label}を入力してください。`;
  try {
    fromJson(schema, { [field]: value } satisfies JsonObject);
    return undefined;
  } catch {
    return `${label}は数字で入力してください。`;
  }
}

/// 前後の空白を除き、全角数字を半角にする
function normalize(text: string): string {
  return text.trim().replace(/[０-９]/g, (c) => String.fromCharCode(c.charCodeAt(0) - 0xfee0));
}
