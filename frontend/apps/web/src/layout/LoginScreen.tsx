import { Alert, Button, pageClass } from "@platform/ui";

/// ログインしていないときの画面
export function LoginScreen({ notice, onSignIn }: { notice: string | null; onSignIn: () => void }) {
  return (
    <main className={pageClass()}>
      <h1>給与管理</h1>
      {notice && <Alert>{notice}</Alert>}
      <Button onClick={onSignIn}>ログイン</Button>
    </main>
  );
}
