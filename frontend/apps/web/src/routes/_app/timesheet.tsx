import { timesheet } from "@platform/api-client";
import { createFileRoute } from "@tanstack/react-router";

import { NotRegisteredAsStaff } from "../../features/payroll/MyPayslips";
import { monthSearch, thisMonth } from "../../features/timesheet/month";
import { MyTimesheetPage } from "../../features/timesheet/MyTimesheet";
import { ensure } from "../../lib/loaders";

/// 派遣社員の勤怠。対象月は URL の引数で選ぶ(指定がなければ今月)
export const Route = createFileRoute("/_app/timesheet")({
  staticData: { system: "timesheet" },
  validateSearch: monthSearch,
  loaderDeps: ({ search }) => search,
  loader: async ({ context, deps, location }) => {
    // 今日の日付は画面を描く前に決める
    const month =
      deps.year !== undefined && deps.month !== undefined
        ? { year: deps.year, month: deps.month }
        : thisMonth(new Date());
    if (context.me.staff) {
      await Promise.all([
        ensure(context, timesheet.TimesheetService.method.getMyTimesheet, month, location.href),
        ensure(context, timesheet.TimesheetService.method.listProjects, {}, location.href),
      ]);
    }
    return { month };
  },
  component: TimesheetRoute,
});

function TimesheetRoute() {
  const { me } = Route.useRouteContext();
  const { month } = Route.useLoaderData();
  const navigate = Route.useNavigate();
  if (!me.staff) return <NotRegisteredAsStaff />;
  return (
    <MyTimesheetPage
      // 月を変えたら、入力は選んだ月の勤務表から始め直す
      key={`${month.year}-${month.month}`}
      month={month}
      onChangeMonth={(next) => void navigate({ search: { year: next.year, month: next.month } })}
    />
  );
}
