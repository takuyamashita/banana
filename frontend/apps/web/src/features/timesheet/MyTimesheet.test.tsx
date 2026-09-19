import { create } from "@bufbuild/protobuf";
import { type ConnectRouter } from "@connectrpc/connect";
import { timesheet } from "@platform/api-client";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { renderWithApi } from "../../test-utils";
import { MyTimesheetPage } from "./MyTimesheet";

const projects = [
  { projectId: 10n, name: "案件A" },
  { projectId: 11n, name: "案件B" },
];

interface Entry {
  date: string;
  projectId: bigint;
  projectName: string;
  workMinutes: number;
}

interface State {
  timesheetId: bigint;
  status: timesheet.TimesheetStatus;
  returnedReason: string;
  entries: Entry[];
}

/// このテストの中だけの勤務表の置き場。保存・申告の要求を覚えておく。応答は毎回、今の状態から作る
function routes(options: { initial?: Partial<State> } = {}) {
  const state: State = {
    timesheetId: 0n,
    status: timesheet.TimesheetStatus.DRAFT,
    returnedReason: "",
    entries: [],
    ...options.initial,
  };
  const current = () =>
    create(timesheet.TimesheetSchema, {
      ...state,
      staffId: 3n,
      staffName: "派遣 太郎",
      year: 2026,
      month: 9,
      entries: state.entries.map((e) => ({ ...e })),
      totalMinutes: state.entries.reduce((sum, e) => sum + e.workMinutes, 0),
    });
  const saved: timesheet.SaveMyTimesheetRequest[] = [];
  const handler = ({ service }: ConnectRouter) => {
    service(timesheet.TimesheetService, {
      getMyTimesheet: () => ({ timesheet: current() }),
      listProjects: () => ({ projects }),
      saveMyTimesheet: (req) => {
        saved.push(req);
        state.timesheetId = 1n;
        state.entries = req.entries.map((e) => ({
          date: e.date,
          projectId: e.projectId,
          workMinutes: e.workMinutes,
          projectName: projects.find((p) => p.projectId === e.projectId)?.name ?? "",
        }));
        return { timesheet: current() };
      },
      submitMyTimesheet: () => {
        state.status = timesheet.TimesheetStatus.SUBMITTED;
        state.returnedReason = "";
        return { timesheet: current() };
      },
    });
  };
  return { handler, saved };
}

function renderPage(
  r: ReturnType<typeof routes>,
  onChangeMonth = vi.fn<(month: { year: number; month: number }) => void>(),
) {
  renderWithApi(<MyTimesheetPage month={{ year: 2026, month: 9 }} onChangeMonth={onChangeMonth} />, r.handler);
  return onChangeMonth;
}

test("稼働を書いて申告すると、書き直せない申告済みの勤務表になる", async () => {
  const r = routes();
  renderPage(r);
  const user = userEvent.setup();

  await user.click(await screen.findByRole("button", { name: "行を足す" }));
  await user.selectOptions(screen.getByLabelText("1行目の案件"), "10");
  await user.type(screen.getByLabelText("1行目の稼働(分)"), "480");
  await user.click(screen.getByRole("button", { name: "行を足す" }));
  // 足した行は、前の行の案件と次の日で始まる
  expect(screen.getByLabelText("2行目の日付")).toHaveValue("2026-09-02");
  expect(screen.getByLabelText("2行目の案件")).toHaveValue("10");
  await user.type(screen.getByLabelText("2行目の稼働(分)"), "450");
  expect(screen.getByTestId("timesheet-total")).toHaveTextContent("15時間30分");

  await user.click(screen.getByRole("button", { name: "申告する" }));

  // 申告の前に保存する
  expect(await screen.findByText("申告しました。管理者の承認を待っています。")).toBeVisible();
  expect(r.saved).toHaveLength(1);
  expect(r.saved[0]?.entries.map((e) => [e.date, e.workMinutes])).toEqual([
    ["2026-09-01", 480],
    ["2026-09-02", 450],
  ]);
  expect(screen.getByTestId("timesheet-status")).toHaveTextContent("申告済み(承認待ち)");
  expect(screen.queryByRole("button", { name: "行を足す" })).toBeNull();
  expect(screen.getByRole("cell", { name: "7時間30分" })).toBeVisible();
});

test("差し戻された勤務表には理由が出て、直して申告し直せる", async () => {
  renderPage(
    routes({
      initial: {
        timesheetId: 1n,
        returnedReason: "9/2 の案件が違います",
        entries: [{ date: "2026-09-02", projectId: 10n, projectName: "案件A", workMinutes: 450 }],
      },
    }),
  );

  expect(await screen.findByRole("alert")).toHaveTextContent("差し戻されました: 9/2 の案件が違います");
  expect(screen.getByLabelText("1行目の案件")).toHaveValue("10");
});

test("入力が足りなければ送らずに伝え、月を選び直せる", async () => {
  const r = routes();
  const onChangeMonth = renderPage(r);
  const user = userEvent.setup();

  await user.click(await screen.findByRole("button", { name: "行を足す" }));
  await user.click(screen.getByRole("button", { name: "保存する" }));
  expect(screen.getByRole("alert")).toHaveTextContent("1行目の案件を選んでください。");
  expect(r.saved).toHaveLength(0);

  await user.click(screen.getByRole("button", { name: "前の月" }));
  expect(onChangeMonth).toHaveBeenCalledWith({ year: 2026, month: 8 });
});
