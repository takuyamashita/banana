import { createConnectQueryKey, useMutation, useSuspenseQuery } from "@connectrpc/connect-query";
import { timesheet } from "@platform/api-client";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { errorMessage } from "../../lib/errors";

/// 管理者が、申告された勤務表を承認するか差し戻す画面が使う、データと操作
export function useTimesheetApproval() {
  const queryClient = useQueryClient();
  const submitted = useSuspenseQuery(timesheet.TimesheetService.method.listSubmittedTimesheets, {}).data.timesheets;
  /// 勤務表番号 → 入力中の差し戻しの理由
  const [reasons, setReasons] = useState<ReadonlyMap<bigint, string>>(new Map());
  const [notice, setNotice] = useState<string | null>(null);

  const refresh = () =>
    queryClient.invalidateQueries({
      queryKey: createConnectQueryKey({
        schema: timesheet.TimesheetService.method.listSubmittedTimesheets,
        cardinality: "finite",
      }),
    });
  const approve = useMutation(timesheet.TimesheetService.method.approveTimesheet, {
    onSuccess: async ({ timesheet: t }) => {
      setNotice(`${t?.staffName ?? ""}さんの${t?.year}年${t?.month}月の勤務表を承認しました。`);
      await refresh();
    },
  });
  const sendBack = useMutation(timesheet.TimesheetService.method.returnTimesheet, {
    onSuccess: async ({ timesheet: t }) => {
      setNotice(`${t?.staffName ?? ""}さんの${t?.year}年${t?.month}月の勤務表を差し戻しました。`);
      await refresh();
    },
  });
  const clearMessages = () => {
    setNotice(null);
    approve.reset();
    sendBack.reset();
  };
  const serverError = [approve.error, sendBack.error].find((e) => e !== null);

  return {
    submitted,
    reasonOf: (id: bigint) => reasons.get(id) ?? "",
    busy: approve.isPending || sendBack.isPending,
    error: serverError ? errorMessage(serverError) : null,
    notice,
    onReasonChange: (id: bigint, reason: string) => setReasons((current) => new Map(current).set(id, reason)),
    onApprove: (id: bigint) => {
      clearMessages();
      approve.mutate({ timesheetId: id });
    },
    onReturn: (id: bigint) => {
      clearMessages();
      sendBack.mutate({ timesheetId: id, reason: reasons.get(id) ?? "" });
    },
  };
}

export type TimesheetApprovalModel = ReturnType<typeof useTimesheetApproval>;
