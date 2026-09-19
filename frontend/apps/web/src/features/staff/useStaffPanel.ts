import { fromJson } from "@bufbuild/protobuf";
import { createConnectQueryKey, useMutation, useSuspenseQuery } from "@connectrpc/connect-query";
import { CreateStaffRequestSchema, StaffService, type CreateStaffRequestJson } from "@platform/api-client";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { errorMessage } from "../../lib/errors";

/// 登録の入力。要求の JSON 形と同じ項目を、入力されたまま持つ
export type StaffDraft = { [K in keyof Required<CreateStaffRequestJson>]: string };

const emptyDraft: StaffDraft = { email: "", displayName: "", temporaryPassword: "" };

/// 派遣社員の登録と一覧の画面が使う、データと操作
export function useStaffPanel() {
  const queryClient = useQueryClient();
  const staff = useSuspenseQuery(StaffService.method.listStaff, {}).data.staff;
  const [draft, setDraft] = useState(emptyDraft);
  const create = useMutation(StaffService.method.createStaff, {
    onSuccess: async () => {
      setDraft(emptyDraft);
      await queryClient.invalidateQueries({
        queryKey: createConnectQueryKey({ schema: StaffService.method.listStaff, cardinality: "finite" }),
      });
    },
  });

  return {
    staff,
    draft,
    busy: create.isPending,
    error: create.error ? errorMessage(create.error) : null,
    createdId: create.data?.staffId,
    onEdit: (patch: Partial<StaffDraft>) => setDraft((prev) => ({ ...prev, ...patch })),
    onCreate: () => create.mutate(fromJson(CreateStaffRequestSchema, draft)),
  };
}

export type StaffPanelModel = ReturnType<typeof useStaffPanel>;
