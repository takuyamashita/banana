import { fromJson } from "@bufbuild/protobuf";
import { createConnectQueryKey, useMutation, useSuspenseQuery } from "@connectrpc/connect-query";
import { CreateProjectRequestSchema, ProjectService, type CreateProjectRequestJson } from "@platform/api-client";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { errorMessage } from "../../lib/errors";

/// 登録の入力。要求の JSON 形と同じ項目を、入力されたまま持つ
export type ProjectDraft = { [K in keyof Required<CreateProjectRequestJson>]: string };

/// 案件の登録と一覧の画面が使う、データと操作
export function useProjectPanel() {
  const queryClient = useQueryClient();
  const projects = useSuspenseQuery(ProjectService.method.listProjects, {}).data.projects;
  const [draft, setDraft] = useState<ProjectDraft>({ name: "" });
  const create = useMutation(ProjectService.method.createProject, {
    onSuccess: async () => {
      setDraft({ name: "" });
      await queryClient.invalidateQueries({
        queryKey: createConnectQueryKey({ schema: ProjectService.method.listProjects, cardinality: "finite" }),
      });
    },
  });

  return {
    projects,
    draft,
    busy: create.isPending,
    error: create.error ? errorMessage(create.error) : null,
    onEdit: (patch: Partial<ProjectDraft>) => setDraft((prev) => ({ ...prev, ...patch })),
    onCreate: () => create.mutate(fromJson(CreateProjectRequestSchema, draft)),
  };
}

export type ProjectPanelModel = ReturnType<typeof useProjectPanel>;
