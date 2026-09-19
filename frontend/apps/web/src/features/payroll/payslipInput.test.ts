import { draftReducer, initialDraft, toCreateRequest, type PayslipDraft } from "./payslipInput";

const filled = (): PayslipDraft => ({
  ...initialDraft({ year: 2026, month: 8 }),
  lines: [{ key: 0, projectId: "10", workMinutes: "600", hourlyRate: "1200" }],
});

test("入力を、生成したスキーマで要求の型にする", () => {
  const result = toCreateRequest("1", filled());

  expect(result).toMatchObject({
    request: {
      staffId: 1n,
      payYear: 2026,
      payMonth: 8,
      lines: [{ projectId: 10n, workMinutes: 600, hourlyRate: 1200n }],
    },
  });
});

test("全角の数字と前後の空白は、そのまま読める", () => {
  const draft = { ...filled(), payMonth: " ９ " };
  expect(toCreateRequest("1", draft)).toMatchObject({ request: { payMonth: 9 } });
});

test("読めない入力は、どの項目かを添えて断る", () => {
  const line = { key: 0, projectId: "10", workMinutes: "1.5", hourlyRate: "1200" };
  expect(toCreateRequest(undefined, filled())).toEqual({ error: "派遣社員を選んでください。" });
  expect(toCreateRequest("1", { ...filled(), payYear: "" })).toEqual({ error: "年を入力してください。" });
  expect(toCreateRequest("1", { ...filled(), lines: [line] })).toEqual({
    error: "明細 1の稼働(分)は数字で入力してください。",
  });
  expect(toCreateRequest("1", { ...filled(), lines: [{ ...line, workMinutes: "600", projectId: "" }] })).toEqual({
    error: "明細 1の案件を選んでください。",
  });
});

test("行を消しても他の行はそのままで、新しい行には使っていない key を振る", () => {
  let draft = draftReducer(filled(), { type: "addLine" });
  draft = draftReducer(draft, { type: "setLine", key: 1, field: "workMinutes", value: "120" });
  draft = draftReducer(draft, { type: "removeLine", key: 0 });

  expect(draft.lines).toEqual([{ key: 1, projectId: "", workMinutes: "120", hourlyRate: "" }]);
  expect(draftReducer(draft, { type: "clearLines" }).lines).toEqual([
    { key: 2, projectId: "", workMinutes: "", hourlyRate: "" },
  ]);
});
