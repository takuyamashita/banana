import { createConnectQueryKey, useMutation, useQuery } from "@connectrpc/connect-query";
import { ProjectService } from "@platform/api-client";
import { Alert, Button, Card, Field } from "@platform/ui";
import { useQueryClient } from "@tanstack/react-query";
import { useState, type FormEvent } from "react";

import { errorMessage } from "../../lib/errors";

/// 管理者が案件を登録・一覧する画面
export function ProjectPanel() {
  const queryClient = useQueryClient();
  const projects = useQuery(ProjectService.method.listProjects, {});
  const [name, setName] = useState("");
  const createProject = useMutation(ProjectService.method.createProject, {
    onSuccess: async () => {
      setName("");
      await queryClient.invalidateQueries({
        queryKey: createConnectQueryKey({ schema: ProjectService.method.listProjects, cardinality: "finite" }),
      });
    },
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    createProject.mutate({ name });
  }

  return (
    <Card title="案件">
      <form onSubmit={submit} className="row" aria-label="案件の登録">
        <Field label="案件名" value={name} onChange={(e) => setName(e.target.value)} maxLength={100} required />
        <Button type="submit" disabled={createProject.isPending}>
          登録する
        </Button>
      </form>
      {createProject.error && <Alert>{errorMessage(createProject.error)}</Alert>}
      {projects.error && <Alert>{errorMessage(projects.error)}</Alert>}
      {projects.isPending && <output>読み込み中…</output>}
      <ul aria-label="登録済みの案件">
        {projects.data?.projects.map((p) => (
          <li key={String(p.projectId)}>
            #{String(p.projectId)} {p.name}
          </li>
        ))}
      </ul>
    </Card>
  );
}
