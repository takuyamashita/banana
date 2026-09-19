import { fromJson, type DescMessage, type JsonObject } from "@bufbuild/protobuf";

/// 入力欄の中身。要求の JSON 形と同じ項目を、入力されたまま(文字列で)持つ
export type Fields<T> = { [K in keyof Required<T>]: string };

/// 1つの項目が、スキーマのその項目の型として読めるか。読めなければ文言を返す
/// (業務の決まり(15分単位など)はサーバーが確かめるので、ここでは型として読めるかだけを見る)
export function checkField(schema: DescMessage, field: string, text: string, label: string): string | undefined {
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
export function normalize(text: string): string {
  return text.trim().replace(/[０-９]/g, (c) => String.fromCharCode(c.charCodeAt(0) - 0xfee0));
}
