import type { Project } from "@platform/api-client";
import { Alert, Button, Card, Field } from "@platform/ui";
import { useCallback, useEffect, useState, type FormEvent } from "react";

import { errorMessage, useApi } from "../../lib/api";

/// 管理者が案件を登録・一覧する画面
export function ProjectPanel() {
  const api = useApi();
  const [projects, setProjects] = useState<Project[]>([]);
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(() => {
    api.project
      .listProjects({})
      .then((res) => setProjects(res.projects))
      .catch((err: unknown) => setError(errorMessage(err)));
  }, [api]);

  useEffect(reload, [reload]);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setError(null);
    try {
      await api.project.createProject({ name });
      setName("");
      reload();
    } catch (err) {
      setError(errorMessage(err));
    }
  }

  return (
    <Card title="案件">
      <form onSubmit={(e) => void submit(e)} className="row">
        <Field label="案件名" value={name} onChange={(e) => setName(e.target.value)} required />
        <Button type="submit">登録する</Button>
      </form>
      {error && <Alert>{error}</Alert>}
      <ul>
        {projects.map((p) => (
          <li key={String(p.projectId)}>
            #{String(p.projectId)} {p.name}
          </li>
        ))}
      </ul>
    </Card>
  );
}
