import { UserManager, WebStorageStateStore, type User } from "oidc-client-ts";

import type { RuntimeConfig } from "./config";

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
export async function signOut(manager: UserManager, config: RuntimeConfig): Promise<void> {
  // 失効に失敗しても(失効の宛先がない IdP など)、IdP のログアウトは続ける
  await manager.revokeTokens(["refresh_token"]).catch(() => undefined);
  const param = config.oidc.postLogoutRedirectParam;
  await manager.signoutRedirect(param ? { extraQueryParams: { [param]: `${window.location.origin}/` } } : undefined);
}

// 認可コードは1回しか交換できない。StrictMode の開発時は effect が2回走るので、
// 同じ UserManager に対する復元処理は最初の Promise を使い回す
const sessions = new WeakMap<UserManager, Promise<User | null>>();

/// ログイン後のリダイレクト(?code=...)なら処理してから、現在のユーザーを返す
export function restoreSession(manager: UserManager): Promise<User | null> {
  let session = sessions.get(manager);
  if (!session) {
    session = restore(manager);
    sessions.set(manager, session);
  }
  return session;
}

async function restore(manager: UserManager): Promise<User | null> {
  const params = new URLSearchParams(window.location.search);
  if (params.has("code") && params.has("state")) {
    const user = await manager.signinRedirectCallback();
    window.history.replaceState({}, "", window.location.pathname);
    return user;
  }
  const user = await manager.getUser();
  return user && !user.expired ? user : null;
}
