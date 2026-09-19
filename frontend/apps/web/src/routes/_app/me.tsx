import { PayrollService } from "@platform/api-client";
import { createFileRoute } from "@tanstack/react-router";

import { MyPayslipsPage, NotRegisteredAsStaff } from "../../features/payroll/MyPayslips";
import { ensure } from "../../lib/loaders";

export const Route = createFileRoute("/_app/me")({
  staticData: { system: "payroll" },
  loader: async ({ context, location }) => {
    const staff = context.me.staff;
    if (staff) await ensure(context, PayrollService.method.listPayslips, { staffId: staff.staffId }, location.href);
  },
  component: MeRoute,
});

function MeRoute() {
  const { me } = Route.useRouteContext();
  return me.staff ? <MyPayslipsPage staffId={me.staff.staffId} /> : <NotRegisteredAsStaff />;
}
