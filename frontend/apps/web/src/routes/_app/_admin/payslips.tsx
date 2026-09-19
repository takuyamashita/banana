import { ProjectService, StaffService } from "@platform/api-client";
import { createFileRoute } from "@tanstack/react-router";

import { previousMonth } from "../../../features/payroll/format";
import { PayslipAdminPage } from "../../../features/payroll/PayslipAdmin";
import { payslipSearch } from "../../../features/payroll/search";
import { ensure } from "../../../lib/loaders";

export const Route = createFileRoute("/_app/_admin/payslips")({
  staticData: { system: "payroll" },
  validateSearch: payslipSearch,
  loader: async ({ context, location }) => {
    await Promise.all([
      ensure(context, StaffService.method.listStaff, {}, location.href),
      ensure(context, ProjectService.method.listProjects, {}, location.href),
    ]);
    // 月初に前の月の分を作るので、前の月を既定にする(今日の日付は画面を描く前に決める)
    return { defaultPeriod: previousMonth(new Date()) };
  },
  component: PayslipsRoute,
});

function PayslipsRoute() {
  const { staffId } = Route.useSearch();
  const { defaultPeriod } = Route.useLoaderData();
  const navigate = Route.useNavigate();
  return (
    <PayslipAdminPage
      staffId={staffId}
      defaultPeriod={defaultPeriod}
      onSelectStaff={(id) => void navigate({ search: id === undefined ? {} : { staffId: id } })}
    />
  );
}
