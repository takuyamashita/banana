import { fromJson } from "@bufbuild/protobuf";
import { createConnectQueryKey, skipToken, useMutation, useQuery, useSuspenseQuery } from "@connectrpc/connect-query";
import {
  ListPayslipsRequestSchema,
  PayrollService,
  ProjectService,
  StaffService,
  type Payslip,
} from "@platform/api-client";
import { useQueryClient } from "@tanstack/react-query";
import { useReducer, useState } from "react";

import { errorMessage } from "../../lib/errors";
import { draftReducer, initialDraft, toCreateRequest, type DraftAction } from "./payslipInput";

export interface PayslipAdminProps {
  /// 選んでいる派遣社員(URL の引数。一覧の要求の JSON 形)
  staffId: string | undefined;
  /// 作る月の既定値
  defaultPeriod: { year: number; month: number };
  onSelectStaff: (staffId: string | undefined) => void;
}

/// 給与明細の作成と確定の画面が使う、データと操作
export function usePayslipAdmin({ staffId, defaultPeriod, onSelectStaff }: PayslipAdminProps) {
  const queryClient = useQueryClient();
  const staff = useSuspenseQuery(StaffService.method.listStaff, {}).data.staff;
  const projects = useSuspenseQuery(ProjectService.method.listProjects, {}).data.projects;
  // 一覧は派遣社員ごとに別に持つ(要求がキーになる)。選び直した直後は、新しい人の一覧が届くまで何も出さない
  const payslips = useQuery(
    PayrollService.method.listPayslips,
    staffId === undefined ? skipToken : fromJson(ListPayslipsRequestSchema, { staffId }),
  );
  const [draft, dispatch] = useReducer(draftReducer, defaultPeriod, initialDraft);
  const [inputError, setInputError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const refreshPayslips = () =>
    queryClient.invalidateQueries({
      queryKey: createConnectQueryKey({ schema: PayrollService.method.listPayslips, cardinality: "finite" }),
    });
  const create = useMutation(PayrollService.method.createPayslip, {
    onSuccess: async ({ payslipId }) => {
      dispatch({ type: "clearLines" });
      setNotice(`給与明細 #${payslipId} を作成しました。内容を確かめて確定してください。`);
      await refreshPayslips();
    },
  });
  const finalize = useMutation(PayrollService.method.finalizePayslip, {
    onSuccess: async (_, { payslipId }) => {
      setNotice(`給与明細 #${payslipId} を確定しました。`);
      await refreshPayslips();
    },
  });

  const clearMessages = () => {
    setInputError(null);
    setNotice(null);
    create.reset();
    finalize.reset();
  };
  const serverError = [payslips.error, create.error, finalize.error].find((e) => e !== null);

  return {
    staff,
    projects,
    staffId,
    staffName: staff.find((s) => String(s.staffId) === staffId)?.displayName ?? "",
    draft,
    payslips: payslips.data?.payslips,
    loading: payslips.isLoading,
    busy: create.isPending || finalize.isPending,
    error: inputError ?? (serverError ? errorMessage(serverError) : null),
    notice,
    onSelectStaff,
    onEdit: (action: DraftAction) => dispatch(action),
    onCreate: () => {
      clearMessages();
      const result = toCreateRequest(staffId, draft);
      if ("error" in result) setInputError(result.error);
      else create.mutate(result.request);
    },
    onFinalize: (payslip: Payslip) => {
      clearMessages();
      finalize.mutate({ payslipId: payslip.payslipId });
    },
  };
}

export type PayslipAdminModel = ReturnType<typeof usePayslipAdmin>;
