import type { Staff } from "@platform/api-client";
import { Alert, Button, Card, Field } from "@platform/ui";
import { useCallback, useEffect, useState, type FormEvent } from "react";

import { errorMessage, useApi } from "../../lib/api";

/// 管理者が派遣社員を登録・一覧する画面。登録すると認証基盤にもユーザーが作られる
export function StaffPanel() {
  const api = useApi();
  const [staff, setStaff] = useState<Staff[]>([]);
  const [email, setEmail] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [created, setCreated] = useState<string | null>(null);

  const reload = useCallback(() => {
    api.staff
      .listStaff({})
      .then((res) => setStaff(res.staff))
      .catch((err: unknown) => setError(errorMessage(err)));
  }, [api]);

  useEffect(reload, [reload]);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setError(null);
    setCreated(null);
    try {
      const res = await api.staff.createStaff({ email, displayName, temporaryPassword: password });
      setCreated(`派遣社員 #${res.staffId} を登録しました。初回ログインでパスワードの変更を求められます。`);
      setEmail("");
      setDisplayName("");
      setPassword("");
      reload();
    } catch (err) {
      setError(errorMessage(err));
    }
  }

  return (
    <Card title="派遣社員">
      <form onSubmit={(e) => void submit(e)}>
        <Field label="メールアドレス" type="email" value={email} onChange={(e) => setEmail(e.target.value)} required />
        <Field label="表示名" value={displayName} onChange={(e) => setDisplayName(e.target.value)} required />
        <Field
          label="仮パスワード"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
        />
        <Button type="submit">登録する</Button>
      </form>
      {error && <Alert>{error}</Alert>}
      {created && <Alert tone="success">{created}</Alert>}
      <table>
        <thead>
          <tr>
            <th className="num">ID</th>
            <th>表示名</th>
            <th>メールアドレス</th>
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
