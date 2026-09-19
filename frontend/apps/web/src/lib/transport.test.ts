import { createRouterTransport } from "@connectrpc/connect";
import { PayrollService, routeByService, timesheet } from "@platform/api-client";
import { createClient } from "@connectrpc/connect";

test("RPC はサービスごとの宛先に届き、知らないサービスは給与に届く", async () => {
  const reached: string[] = [];
  const payroll = createRouterTransport(({ service }) => {
    service(PayrollService, {
      listPayslips: () => {
        reached.push("payroll");
        return { payslips: [] };
      },
    });
  });
  const timesheetServer = createRouterTransport(({ service }) => {
    service(timesheet.TimesheetService, {
      listProjects: () => {
        reached.push("timesheet");
        return { projects: [] };
      },
    });
  });
  const transport = routeByService(new Map([[timesheet.TimesheetService, timesheetServer]]), payroll);

  await createClient(timesheet.TimesheetService, transport).listProjects({});
  await createClient(PayrollService, transport).listPayslips({ staffId: 1n });

  expect(reached).toEqual(["timesheet", "payroll"]);
});
