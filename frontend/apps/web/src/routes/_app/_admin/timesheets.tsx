import { timesheet } from "@platform/api-client";
import { createFileRoute } from "@tanstack/react-router";

import { TimesheetApprovalPage } from "../../../features/timesheet/TimesheetApproval";
import { ensure } from "../../../lib/loaders";

export const Route = createFileRoute("/_app/_admin/timesheets")({
  loader: ({ context, location }) =>
    ensure(context, timesheet.TimesheetService.method.listSubmittedTimesheets, {}, location.href),
  component: TimesheetApprovalPage,
});
