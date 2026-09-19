import { payslipSearch } from "./search";

test("URL の派遣社員は、一覧の要求のスキーマで読めるものだけ受け付ける", () => {
  expect(payslipSearch({ staffId: "3" })).toEqual({ staffId: "3" });
  expect(payslipSearch({ staffId: "abc" })).toEqual({});
  expect(payslipSearch({ staffId: "0" })).toEqual({});
  expect(payslipSearch({})).toEqual({});
});
