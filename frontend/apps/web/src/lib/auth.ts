import { UserManager, WebStorageStateStore, type User } from "oidc-client-ts";

import type { RuntimeConfig } from "./config";
import { errorMessage, SESSION_EXPIRED } from "./errors";

/// Keycloak(ローカル)と Cognito(stg/prd)のどちらも標準の OIDC + PKCE で扱う。差は config.json だけ
export function createUserManager(config: RuntimeConfig): UserManager {
  const { endSessionEndpoint, revocationEndpoint } = config.oidc;
  return new UserManager({
    // Cognito はディスカバリに end_session_endpoint を載せないので、config.json の値で補う
    metadataSeed: {
      ...(endSessionEndpoint ? { end_session_endpoint: endSessionEndpoint } : {}),
      ...(revocationEndpoint ? { revocation_endpoint: revocationEndpoint } : {}),
    },
    authority: config.oidc.authority,
    client_id: config.oidc.clientId,
    redirect_uri: `${window.location.origin}/`,
    post_logout_redirect_uri: `${window.location.origin}/`,
    response_type: "code",
    scope: "openid profile email",
    userStore: new WebStorageStateStore({ store: window.sessionStorage }),
    automaticSilentRenew: true,
  });
}

/// ログアウト。リフレッシュトークンを失効させてから、IdP のセッションも終わらせる。
/// ブラウザのセッションだけを消すと IdP にログインが残り、共用の端末では次の人が
/// パスワードなしで前の人としてログインできてしまう
export async function signOut(
  manager: Pick<UserManager, "revokeTokens" | "signoutRedirect">,
  config: RuntimeConfig,
): Promise<void> {
  // 失効に失敗しても(失効の宛先がない IdP など)、IdP のログアウトは続ける
  await manager.revokeTokens(["refresh_token"]).catch(() => undefined);
  const param = config.oidc.postLogoutRedirectParam;
  await manager.signoutRedirect(param ? { extraQueryParams: { [param]: `${window.location.origin}/` } } : undefined);
}

/// ログイン状態の復元に使う UserManager の操作
type SessionSource = Pick<UserManager, "signinRedirectCallback" | "getUser" | "signinSilent">;

/// ログインから戻ってきたが、ログインできなかった(IdP が断った・認可コードを交換できなかった)
export class SignInError extends Error {
  constructor(cause: unknown) {
    super("ログインできませんでした。もう一度お試しください。", { cause });
  }
}

/// ログイン後のリダイレクトなら処理してから、現在のユーザーを返す。ログインしていなければ null。
/// ログインから戻ってきたときは、URL をログイン前に開こうとしていた画面に置き換える。
/// ルーターが URL を読む前に呼ぶ(画面の移動が始まってから戻り先に移ると、重なった移動に上書きされうる)。
/// 認可コードは1回しか交換できないので、呼ぶのはアプリで1回だけにする(createAuth が結果を使い回す)
export async function restoreSession(manager: SessionSource): Promise<User | null> {
  const params = new URLSearchParams(window.location.search);
  // IdP からの戻りは、成功なら ?code=...&state=...、断られたら ?error=...&state=...
  if (params.has("state") && (params.has("code") || params.has("error"))) {
    let returnTo: string | undefined;
    try {
      const user = await manager.signinRedirectCallback();
      returnTo = returnToOf(user.state);
      return user;
    } catch (err) {
      throw new SignInError(err);
    } finally {
      // 成否によらずコードを URL から消す(失敗したコードで再読み込みのたびに交換し直さない)
      window.history.replaceState({}, "", returnTo ?? window.location.pathname);
    }
  }
  const user = await manager.getUser();
  if (!user) return null;
  if (!user.expired) return user;
  // アクセストークンが切れていても、リフレッシュトークンが生きていればログインし直さずに続ける
  if (!user.refresh_token) return null;
  return manager.signinSilent().catch(() => null);
}

function returnToOf(state: unknown): string | undefined {
  return typeof state === "object" && state !== null && "returnTo" in state ? safeReturnTo(state.returnTo) : undefined;
}

/// ログイン後に戻る先として使ってよいか。同じサイトの中のパス(/ で始まり、// や /\ で始まらない)だけ。
/// 外のサイトに飛ばされる(オープンリダイレクト)のを防ぐ
export function safeReturnTo(value: unknown): string | undefined {
  if (typeof value !== "string" || !value.startsWith("/")) return undefined;
  if (value.startsWith("//") || value.startsWith("/\\")) return undefined;
  return value;
}

/// ログインの状態。画面は useSyncExternalStore でこれを購読する
export interface Session {
  user: User | null;
  /// ログイン画面に出す知らせ(セッションが切れた・ログインできなかった)
  notice: string | null;
}

/// ログインの状態を持つ外部ストア。アプリの初期化で1つだけ作る
export interface Auth {
  subscribe: (listener: () => void) => () => void;
  getSnapshot: () => Session;
  /// ログインの状態を確かめる。初回はログインからの戻りも処理する。ログインしていなければ null
  restore: () => Promise<User | null>;
  accessToken: () => Promise<string | undefined>;
  signIn: (returnTo: string | undefined) => Promise<void>;
  signOut: () => Promise<void>;
  /// ログインを終わらせ、ログイン画面に知らせを出す
  expire: (notice: string) => void;
}

/// ログインの状態を持つ外部ストアが使う UserManager の操作
export type AuthManager = SessionSource &
  Pick<UserManager, "removeUser" | "signinRedirect" | "revokeTokens" | "signoutRedirect"> & {
    events: Pick<UserManager["events"], "addAccessTokenExpired" | "addSilentRenewError" | "addUserLoaded">;
  };

/// ログインの状態を持つ外部ストアを作る。`onExpired` はログインが切れたときに呼ぶ(データの破棄と画面の移動)
export function createAuth(manager: AuthManager, config: RuntimeConfig, onExpired: () => void): Auth {
  let session: Session = { user: null, notice: null };
  let restored: Promise<void> | undefined;
  const listeners = new Set<() => void>();
  const set = (next: Session) => {
    session = next;
    for (const listener of listeners) listener();
  };
  const expire = (notice: string) => {
    void manager.removeUser();
    set({ user: null, notice });
    onExpired();
  };
  // トークンが切れた・更新できなかったら、ログインし直してもらう
  manager.events.addAccessTokenExpired(() => expire(SESSION_EXPIRED));
  manager.events.addSilentRenewError(() => expire(SESSION_EXPIRED));
  // 自動更新で新しいトークンになった
  manager.events.addUserLoaded((user) => set({ user, notice: null }));

  return {
    subscribe: (listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    getSnapshot: () => session,
    restore: async () => {
      restored ??= restoreSession(manager).then(
        (user) => set({ user, notice: null }),
        (err: unknown) => set({ user: null, notice: err instanceof SignInError ? err.message : errorMessage(err) }),
      );
      await restored;
      return session.user;
    },
    accessToken: async () => (await manager.getUser())?.access_token,
    signIn: (to) => manager.signinRedirect(to === undefined ? undefined : { state: { returnTo: to } }),
    signOut: () => signOut(manager, config),
    expire,
  };
}
