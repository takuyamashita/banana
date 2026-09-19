import { Code, ConnectError } from "@platform/api-client";

/// API のエラーを画面に出す文言にする。
///
/// サーバーが利用者に向けて書いた文言(入力の誤り・重複・権限など)はそのまま出す。
/// それ以外(つながらない・サーバー内部の失敗・想定外)は、原因の英語や内部の詳細を出さずに定型文にする
export function errorMessage(err: unknown): string {
  if (!(err instanceof ConnectError)) return UNEXPECTED;
  switch (err.code) {
    case Code.InvalidArgument:
    case Code.AlreadyExists:
    case Code.NotFound:
    case Code.FailedPrecondition:
    case Code.PermissionDenied:
      return err.rawMessage;
    case Code.Unauthenticated:
      return SESSION_EXPIRED;
    case Code.Unavailable:
    case Code.DeadlineExceeded:
    case Code.ResourceExhausted:
      return "サーバーに接続できないか、混み合っています。時間をおいてもう一度お試しください。";
    default:
      return UNEXPECTED;
  }
}

export const SESSION_EXPIRED = "ログインの有効期限が切れました。もう一度ログインしてください。";

const UNEXPECTED = "予期しないエラーが発生しました。続く場合は管理者に連絡してください。";

/// 時間をおけば通るかもしれないエラーか(自動で取り直してよいか)
export function isTransient(err: unknown): boolean {
  return (
    err instanceof ConnectError &&
    [Code.Unavailable, Code.DeadlineExceeded, Code.ResourceExhausted, Code.Unknown].includes(err.code)
  );
}
