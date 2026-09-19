# 0004 画面のデータ取得に TanStack Query と connect-query を使う

- 日付: 2026-09-19
- 状態: 採用

## 決定

画面は API を `useEffect` と `useState` で呼ばず、TanStack Query の `useQuery`・`useMutation` を connect-query 経由で使う。proto のメソッド記述子(`PayrollService.method.listPayslips`)をそのまま渡し、更新した後は `createConnectQueryKey` で一覧を取り直す。api-client は gRPC-Web の transport を公開し、画面は `TransportProvider` で受け取る。

## 理由

自前で書くと、古い応答が新しい応答を上書きする・読み込み中の表示・二重送信の防止・ログアウト時のデータの破棄を画面ごとに書くことになる(実際に、派遣社員を選び直した直後に前の人の給与明細と確定ボタンが出る不具合があった)。キーは proto のメソッドと入力から決まるので、取り直す範囲を名前で間違えない。取り直すのは一時的なエラーだけにし、Unauthenticated はどの取得・更新でもログイン画面に戻す(`lib/query.ts`)。
