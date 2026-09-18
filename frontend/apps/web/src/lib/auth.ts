import { UserManager, WebStorageStateStore, type User } from "oidc-client-ts";

import type { RuntimeConfig } from "./config";

/// Keycloak(ローカル)と Cognito(stg/prd)のどちらも標準の OIDC + PKCE で扱う。差は config.json だけ
export function createUserManager(config: RuntimeConfig): UserManager {
  return new UserManager({
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

/// ログアウト。Cognito は OIDC ディスカバリに end_session_endpoint を載せないので
/// signoutRedirect が失敗する。その場合はローカルのセッションだけ消して戻る
export async function signOut(manager: UserManager): Promise<void> {
  try {
    await manager.signoutRedirect();
  } catch {
    await manager.removeUser();
    window.location.assign("/");
  }
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
