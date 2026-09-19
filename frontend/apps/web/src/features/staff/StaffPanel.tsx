import { createConnectQueryKey, useMutation, useQuery } from "@connectrpc/connect-query";
import { StaffService } from "@platform/api-client";
import { Alert, Button, Card, Field } from "@platform/ui";
import { useQueryClient } from "@tanstack/react-query";
import { useState, type FormEvent } from "react";

import { errorMessage } from "../../lib/errors";

const emptyForm = { email: "", displayName: "", temporaryPassword: "" };

/// 管理者が派遣社員を登録・一覧する画面。登録すると認証基盤にもユーザーが作られる
export function StaffPanel() {
  const queryClient = useQueryClient();
  const staff = useQuery(StaffService.method.listStaff, {});
  const [form, setForm] = useState(emptyForm);
  const createStaff = useMutation(StaffService.method.createStaff, {
    onSuccess: async () => {
      setForm(emptyForm);
      await queryClient.invalidateQueries({
        queryKey: createConnectQueryKey({ schema: StaffService.method.listStaff, cardinality: "finite" }),
      });
    },
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    createStaff.mutate(form);
  }

  return (
    <Card title="派遣社員">
      <form onSubmit={submit} aria-label="派遣社員の登録">
        <Field
          label="メールアドレス"
          type="email"
          autoComplete="off"
          value={form.email}
          onChange={(e) => setForm({ ...form, email: e.target.value })}
          required
        />
        <Field
          label="表示名"
          value={form.displayName}
          onChange={(e) => setForm({ ...form, displayName: e.target.value })}
          required
        />
        <Field
          label="仮パスワード"
          type="password"
          autoComplete="new-password"
          value={form.temporaryPassword}
          onChange={(e) => setForm({ ...form, temporaryPassword: e.target.value })}
          required
        />
        <Button type="submit" disabled={createStaff.isPending}>
          登録する
        </Button>
      </form>
      {createStaff.error && <Alert>{errorMessage(createStaff.error)}</Alert>}
      {createStaff.data && (
        <Alert tone="success">
          派遣社員 #{String(createStaff.data.staffId)} を登録しました。初回ログインでパスワードの変更を求められます。
        </Alert>
      )}
      {staff.error && <Alert>{errorMessage(staff.error)}</Alert>}
      {staff.isPending && <output>読み込み中…</output>}
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
          {staff.data?.staff.map((s) => (
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
