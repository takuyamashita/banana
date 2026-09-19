import { StaffService } from "@platform/api-client";
import { createFileRoute } from "@tanstack/react-router";

import { StaffPage } from "../../../features/staff/StaffPanel";
import { ensure } from "../../../lib/loaders";

export const Route = createFileRoute("/_app/_admin/staff")({
  staticData: { system: "payroll" },
  loader: ({ context, location }) => ensure(context, StaffService.method.listStaff, {}, location.href),
  component: StaffPage,
});
