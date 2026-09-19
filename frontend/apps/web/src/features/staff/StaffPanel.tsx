import { Alert, Button, Card, Field } from "@platform/ui";

import { useStaffPanel, type StaffPanelModel } from "./useStaffPanel";

/// 管理者が派遣社員を登録・一覧する画面。登録すると認証基盤にもユーザーが作られる
export function StaffPage() {
  return <StaffPanelView {...useStaffPanel()} />;
}

export function StaffPanelView({ staff, draft, busy, error, createdId, onEdit, onCreate }: StaffPanelModel) {
  return (
    <Card title="派遣社員">
      <form
        aria-label="派遣社員の登録"
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
          label="表示名"
          value={draft.displayName}
          onChange={(e) => onEdit({ displayName: e.target.value })}
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
          登録する
        </Button>
      </form>
      {error && <Alert>{error}</Alert>}
      {createdId !== undefined && (
        <Alert tone="success">
          派遣社員 #{String(createdId)} を登録しました。初回ログインでパスワードの変更を求められます。
        </Alert>
      )}
      <table aria-label="登録済みの派遣社員">
        <thead>
          <tr>
            <th scope="col" className="num">
              ID
            </th>
            <th scope="col">表示名</th>
            <th scope="col">メールアドレス</th>
          </tr>
        </thead>
        <tbody>
          {staff.map((s) => (
            <tr key={String(s.staffId)}>
              <td className="num">{String(s.staffId)}</td>
              <td>{s.displayName}</td>
              <td>{s.email}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </Card>
  );
}
