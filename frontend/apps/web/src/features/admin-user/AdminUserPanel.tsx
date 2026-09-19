import { Alert, Button, Card, Field } from "@platform/ui";

import { useAdminUserPanel, type AdminUserPanelModel } from "./useAdminUserPanel";

/// 管理者が、もう1人の管理者を追加する画面。認証基盤にログイン用アカウントを作る
export function AdminUserPage() {
  return <AdminUserPanelView {...useAdminUserPanel()} />;
}

export function AdminUserPanelView({ draft, busy, error, addedEmail, onEdit, onCreate }: AdminUserPanelModel) {
  return (
    <Card title="管理者">
      <form
        aria-label="管理者の追加"
        onSubmit={(event) => {
          event.preventDefault();
          onCreate();
        }}
      >
        <Field
          label="メールアドレス"
          type="email"
          autoComplete="off"
          value={draft.email}
          onChange={(e) => onEdit({ email: e.target.value })}
          required
        />
        <Field
          label="仮パスワード"
          type="password"
          autoComplete="new-password"
          value={draft.temporaryPassword}
          onChange={(e) => onEdit({ temporaryPassword: e.target.value })}
          required
        />
        <Button type="submit" disabled={busy}>
          追加する
        </Button>
      </form>
      {error && <Alert>{error}</Alert>}
      {addedEmail !== null && (
        <Alert tone="success">{addedEmail} を管理者にしました。初回ログインでパスワードの変更を求められます。</Alert>
      )}
    </Card>
  );
}
