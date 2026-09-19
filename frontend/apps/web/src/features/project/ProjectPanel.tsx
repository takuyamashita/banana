import { Alert, Button, Card, Field, rowClass } from "@platform/ui";

import { useProjectPanel, type ProjectPanelModel } from "./useProjectPanel";

/// 管理者が案件を登録・一覧する画面
export function ProjectPage() {
  return <ProjectPanelView {...useProjectPanel()} />;
}

export function ProjectPanelView({ projects, draft, busy, error, onEdit, onCreate }: ProjectPanelModel) {
  return (
    <Card title="案件">
      <form
        className={rowClass()}
        aria-label="案件の登録"
        onSubmit={(event) => {
          event.preventDefault();
          onCreate();
        }}
      >
        <Field
          label="案件名"
          value={draft.name}
          onChange={(e) => onEdit({ name: e.target.value })}
          maxLength={100}
          required
        />
        <Button type="submit" disabled={busy}>
          登録する
        </Button>
      </form>
      {error && <Alert>{error}</Alert>}
      <ul aria-label="登録済みの案件">
        {projects.map((p) => (
          <li key={String(p.projectId)}>
            #{String(p.projectId)} {p.name}
          </li>
        ))}
      </ul>
    </Card>
  );
}
