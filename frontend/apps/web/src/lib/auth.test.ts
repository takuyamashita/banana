import { User, type UserManager } from "oidc-client-ts";

import { restoreSession, SignInError } from "./auth";

type Manager = Pick<UserManager, "signinRedirectCallback" | "getUser" | "signinSilent">;

/// 有効期限が切れているかどうかと、リフレッシュトークンを持つかだけを決めたユーザー
function user(options: { expired: boolean; refreshToken?: string }): User {
  const now = Math.floor(Date.now() / 1000);
  return new User({
    access_token: "access",
    token_type: "Bearer",
    profile: { sub: "u1", iss: "https://id.example.com", aud: "web", exp: now + 300, iat: now },
    expires_at: options.expired ? now - 60 : now + 300,
    ...(options.refreshToken ? { refresh_token: options.refreshToken } : {}),
  });
}

function fakeManager(overrides: Partial<Manager> = {}) {
  const signinRedirectCallback = vi.fn<Manager["signinRedirectCallback"]>(async () => user({ expired: false }));
  const getUser = vi.fn<Manager["getUser"]>(async () => null);
  const signinSilent = vi.fn<Manager["signinSilent"]>(async () => null);
  const manager: Manager = { signinRedirectCallback, getUser, signinSilent, ...overrides };
  return { manager, signinRedirectCallback, signinSilent };
}

afterEach(() => window.history.replaceState({}, "", "/"));

test("ログインからの戻りでは認可コードを1回だけ交換し、URL から消す", async () => {
  window.history.replaceState({}, "", "/?code=abc&state=xyz");
  const { manager, signinRedirectCallback } = fakeManager();

  // StrictMode の開発時は effect が2回走る
  await Promise.all([restoreSession(manager), restoreSession(manager)]);

  expect(signinRedirectCallback).toHaveBeenCalledOnce();
  expect(window.location.search).toBe("");
});

test("IdP に断られて戻ってきたら、ログインできなかったと伝えて URL から消す", async () => {
  window.history.replaceState({}, "", "/?error=access_denied&state=xyz");
  const { manager } = fakeManager({
    signinRedirectCallback: async () => {
      throw new Error("access_denied");
    },
  });

  await expect(restoreSession(manager)).rejects.toBeInstanceOf(SignInError);
  expect(window.location.search).toBe("");
});

test("アクセストークンが切れていても、リフレッシュトークンがあればログインし直さずに続ける", async () => {
  const renewed = user({ expired: false });
  const { manager } = fakeManager({
    getUser: async () => user({ expired: true, refreshToken: "refresh" }),
    signinSilent: async () => renewed,
  });

  await expect(restoreSession(manager)).resolves.toBe(renewed);
});

test("アクセストークンが切れていてリフレッシュトークンもなければ、ログインしていないとみなす", async () => {
  const { manager, signinSilent } = fakeManager({ getUser: async () => user({ expired: true }) });

  await expect(restoreSession(manager)).resolves.toBeNull();
  expect(signinSilent).not.toHaveBeenCalled();
});
