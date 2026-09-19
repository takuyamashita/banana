import { fromJson } from "@bufbuild/protobuf";
import { useMutation } from "@connectrpc/connect-query";
import { CreateAdminUserRequestSchema, UserService, type CreateAdminUserRequestJson } from "@platform/api-client";
import { useState } from "react";

import { errorMessage } from "../../lib/errors";

/// 追加の入力。要求の JSON 形と同じ項目を、入力されたまま持つ
export type AdminUserDraft = { [K in keyof Required<CreateAdminUserRequestJson>]: string };

const emptyDraft: AdminUserDraft = { email: "", temporaryPassword: "" };

/// 管理者を追加する画面が使う、データと操作。
/// 作るのは認証基盤のアカウントだけなので、一覧にするものがこちらにはない(一覧は出さない)
export function useAdminUserPanel() {
  const [draft, setDraft] = useState(emptyDraft);
  const create = useMutation(UserService.method.createAdminUser, {
    onSuccess: () => setDraft(emptyDraft),
  });

  return {
    draft,
    busy: create.isPending,
    error: create.error ? errorMessage(create.error) : null,
    // 知らせるのは、認証基盤が振った利用者IDより分かる「誰を管理者にしたか」。
    // 次の追加を始めると data は消えるので、前回の結果が残ったままにならない
    addedEmail: create.data ? (create.variables?.email ?? null) : null,
    onEdit: (patch: Partial<AdminUserDraft>) => setDraft((prev) => ({ ...prev, ...patch })),
    onCreate: () => create.mutate(fromJson(CreateAdminUserRequestSchema, draft)),
  };
}

export type AdminUserPanelModel = ReturnType<typeof useAdminUserPanel>;
