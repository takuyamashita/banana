import { create } from "@bufbuild/protobuf";
import { createConnectQueryKey, useMutation, useSuspenseQuery } from "@connectrpc/connect-query";
import { timesheet } from "@platform/api-client";
import { useQueryClient } from "@tanstack/react-query";
import { useReducer, useState } from "react";

import { errorMessage } from "../../lib/errors";
import { monthLabel, monthRange, shiftMonth, type TimesheetMonth } from "./month";
import {
  draftReducer,
  draftTotalMinutes,
  initialDraft,
  nextEntryDate,
  toSaveRequest,
  type DraftAction,
} from "./timesheetInput";

export interface MyTimesheetProps {
  /// 対象月(URL の引数。勤務表の取得の要求の JSON 形)
  month: TimesheetMonth;
  onChangeMonth: (month: TimesheetMonth) => void;
}

/// 派遣社員が自分の月の勤務表を書いて申告する画面が使う、データと操作。
/// 入力は画面を開いたときの勤務表から始まる(月を変えたら、画面ごと作り直す)
export function useMyTimesheet({ month, onChangeMonth }: MyTimesheetProps) {
  const queryClient = useQueryClient();
  const sheet =
    useSuspenseQuery(timesheet.TimesheetService.method.getMyTimesheet, month).data.timesheet ??
    create(timesheet.TimesheetSchema);
  const projects = useSuspenseQuery(timesheet.TimesheetService.method.listProjects, {}).data.projects;
  const [draft, dispatch] = useReducer(draftReducer, sheet, initialDraft);
  const [inputError, setInputError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const refresh = () =>
    queryClient.invalidateQueries({
      queryKey: createConnectQueryKey({
        schema: timesheet.TimesheetService.method.getMyTimesheet,
        cardinality: "finite",
      }),
    });
  const save = useMutation(timesheet.TimesheetService.method.saveMyTimesheet, {
    onSuccess: async () => {
      setNotice("保存しました。");
      await refresh();
    },
  });
  const submit = useMutation(timesheet.TimesheetService.method.submitMyTimesheet, {
    onSuccess: async () => {
      setNotice("申告しました。管理者の承認を待っています。");
      await refresh();
    },
  });

  const editable = sheet.status === timesheet.TimesheetStatus.DRAFT;
  const range = monthRange(month);
  const clearMessages = () => {
    setInputError(null);
    setNotice(null);
    save.reset();
    submit.reset();
  };
  /// 入力を確かめて保存し、`then` があれば続ける(申告は、保存してから申告する)
  const saveThen = (then?: () => void) => {
    clearMessages();
    const result = toSaveRequest(month, draft);
    if ("error" in result) {
      setInputError(result.error);
      return;
    }
    save.mutate(result.request, then === undefined ? undefined : { onSuccess: then });
  };
  const serverError = [save.error, submit.error].find((e) => e !== null);

  return {
    title: `${monthLabel(month)}の勤務表`,
    staffName: sheet.staffName,
    status: sheet.status,
    /// 差し戻されて直しているときの理由
    returnedReason: editable && sheet.returnedReason !== "" ? sheet.returnedReason : null,
    editable,
    /// 申告・承認した後は、サーバーの勤務表をそのまま見せる
    entries: sheet.entries,
    projects,
    draft,
    range,
    totalMinutes: editable ? draftTotalMinutes(draft) : sheet.totalMinutes,
    busy: save.isPending || submit.isPending,
    error: inputError ?? (serverError ? errorMessage(serverError) : null),
    notice,
    onEdit: (action: DraftAction) => dispatch(action),
    onAddEntry: () => dispatch({ type: "addEntry", date: nextEntryDate(draft, range) }),
    onSave: () => saveThen(),
    onSubmit: () => saveThen(() => submit.mutate(month)),
    onPreviousMonth: () => onChangeMonth(shiftMonth(month, -1)),
    onNextMonth: () => onChangeMonth(shiftMonth(month, 1)),
  };
}

export type MyTimesheetModel = ReturnType<typeof useMyTimesheet>;
