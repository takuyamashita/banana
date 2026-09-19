import { create } from "@bufbuild/protobuf";
import { type ConnectRouter } from "@connectrpc/connect";
import { timesheet } from "@platform/api-client";
import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { renderWithApi } from "../../test-utils";
import { TimesheetApprovalPage } from "./TimesheetApproval";

const submitted = (timesheetId: bigint, staffName: string) =>
  create(timesheet.TimesheetSchema, {
    timesheetId,
    staffName,
    year: 2026,
    month: 9,
    status: timesheet.TimesheetStatus.SUBMITTED,
    entries: [{ date: "2026-09-01", projectId: 10n, projectName: "案件A", workMinutes: 480 }],
    totalMinutes: 480,
  });

function routes() {
  let waiting = [submitted(1n, "派遣 太郎"), submitted(2n, "派遣 花子")];
  const returned: timesheet.ReturnTimesheetRequest[] = [];
  const handler = ({ service }: ConnectRouter) => {
    service(timesheet.TimesheetService, {
      listSubmittedTimesheets: () => ({ timesheets: waiting }),
      approveTimesheet: ({ timesheetId }) => {
        const sheet = waiting.find((t) => t.timesheetId === timesheetId);
        waiting = waiting.filter((t) => t.timesheetId !== timesheetId);
        return { timesheet: sheet };
      },
      returnTimesheet: (req) => {
        returned.push(req);
        const sheet = waiting.find((t) => t.timesheetId === req.timesheetId);
        waiting = waiting.filter((t) => t.timesheetId !== req.timesheetId);
        return { timesheet: sheet };
      },
    });
  };
  return { handler, returned };
}

test("承認すると一覧から消え、差し戻しには理由が付く", async () => {
  const r = routes();
  renderWithApi(<TimesheetApprovalPage />, r.handler);
  const user = userEvent.setup();

  const taro = await screen.findByRole("region", { name: "派遣 太郎さんの2026年9月の勤務表" });
  expect(within(taro).getByRole("cell", { name: "8時間" })).toBeVisible();
  await user.click(within(taro).getByRole("button", { name: "承認する" }));
  expect(await screen.findByText("派遣 太郎さんの2026年9月の勤務表を承認しました。")).toBeVisible();
  expect(screen.queryByRole("region", { name: "派遣 太郎さんの2026年9月の勤務表" })).toBeNull();

  const hanako = screen.getByRole("region", { name: "派遣 花子さんの2026年9月の勤務表" });
  await user.type(within(hanako).getByLabelText("差し戻すときの理由"), "9/1 は休みのはず");
  await user.click(within(hanako).getByRole("button", { name: "差し戻す" }));
  expect(await screen.findByText("承認を待っている勤務表はありません。")).toBeVisible();
  expect(r.returned.map((req) => [req.timesheetId, req.reason])).toEqual([[2n, "9/1 は休みのはず"]]);
});
