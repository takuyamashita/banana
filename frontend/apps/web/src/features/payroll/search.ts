import { fromJson, toJson } from "@bufbuild/protobuf";
import { ListPayslipsRequestSchema, type ListPayslipsRequestJson } from "@platform/api-client";

/// 給与明細の画面の URL の引数。一覧の要求(ListPayslipsRequest)の JSON 形のうち、派遣社員だけを使う
export type PayslipSearch = Pick<ListPayslipsRequestJson, "staffId">;

/// URL の引数を、一覧の要求のスキーマで読む。読めないもの(数字でない番号など)は、選んでいないことにする
export function payslipSearch(raw: Record<string, unknown>): PayslipSearch {
  const staffId = raw["staffId"];
  if (typeof staffId !== "string") return {};
  try {
    const json = toJson(ListPayslipsRequestSchema, fromJson(ListPayslipsRequestSchema, { staffId }));
    return json.staffId === undefined ? {} : { staffId: json.staffId };
  } catch {
    return {};
  }
}
