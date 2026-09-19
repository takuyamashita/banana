import { User, type UserManager } from "oidc-client-ts";

import type { RuntimeConfig } from "./config";
import { createAuth, restoreSession, safeReturnTo, SignInError, type AuthManager } from "./auth";
import { SESSION_EXPIRED } from "./errors";

type Manager = Pick<UserManager, "signinRedirectCallback" | "getUser" | "signinSilent">;

/// 有効期限が切れているかどうかと、リフレッシュトークンを持つかだけを決めたユーザー
function user(options: { expired: boolean; refreshToken?: string; state?: unknown }): User {
  const now = Math.floor(Date.now() / 1000);
  return new User({
    access_token: "access",
    token_type: "Bearer",
    profile: { sub: "u1", iss: "https://id.example.com", aud: "web", exp: now + 300, iat: now },
    expires_at: options.expired ? now - 60 : now + 300,
    ...(options.refreshToken ? { refresh_token: options.refreshToken } : {}),
    ...(options.state === undefined ? {} : { userState: options.state }),
  });
}

function fakeManager(overrides: Partial<Manager> = {}) {
  const signinRedirectCallback = vi.fn<Manager["signinRedirectCallback"]>(async () => user({ expired: false }));
  const getUser = vi.fn<Manager["getUser"]>(async () => null);
  const signinSilent = vi.fn<Manager["signinSilent"]>(async () => null);
  const manager: Manager = { signinRedirectCallback, getUser, signinSilent, ...overrides };
  return { manager, signinRedirectCallback, signinSilent };
}

/// createAuth に渡せる UserManager(イベントの登録と、使う操作だけを持つ)
function fakeFullManager(manager: Manager): { manager: AuthManager; handlers: Record<string, () => void> } {
  const handlers: Record<string, () => void> = {};
  // 登録された処理を、テストから呼べるように取っておく(引数は使わない)
  const register = (name: string) => (handler: (...args: never[]) => unknown) => {
    handlers[name] = () => void handler();
    return () => undefined;
  };
  return {
    manager: {
      ...manager,
      removeUser: vi.fn<AuthManager["removeUser"]>(async () => undefined),
      signinRedirect: vi.fn<AuthManager["signinRedirect"]>(async () => undefined),
      revokeTokens: vi.fn<AuthManager["revokeTokens"]>(async () => undefined),
      signoutRedirect: vi.fn<AuthManager["signoutRedirect"]>(async () => undefined),
      events: {
        addAccessTokenExpired: register("expired"),
        addSilentRenewError: register("renewError"),
        addUserLoaded: register("loaded"),
      },
    },
    handlers,
  };
}

const config: RuntimeConfig = {
  apiBaseUrl: "http://api",
  timesheetApiBaseUrl: "http://api",
  oidc: { authority: "http://id", clientId: "web" },
};

afterEach(() => window.history.replaceState({}, "", "/"));

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

test("ログインからの戻りでは認可コードを1回だけ交換し、URL をログイン前に開こうとしていた画面にする", async () => {
  window.history.replaceState({}, "", "/?code=abc&state=xyz");
  const signinRedirectCallback = vi.fn<Manager["signinRedirectCallback"]>(async () =>
    user({ expired: false, state: { returnTo: "/payslips?staffId=3" } }),
  );
  const { manager } = fakeManager({ signinRedirectCallback });
  const auth = createAuth(fakeFullManager(manager).manager, config, () => {});

  // 画面の移動が重なっても(ルートの beforeLoad が続けて走っても)、交換は1回
  await Promise.all([auth.restore(), auth.restore()]);

  expect(signinRedirectCallback).toHaveBeenCalledOnce();
  expect(window.location.pathname + window.location.search).toBe("/payslips?staffId=3");
});

test("戻り先がサイトの外なら、戻り先には移らずコードだけ URL から消す", async () => {
  window.history.replaceState({}, "", "/?code=abc&state=xyz");
  const { manager } = fakeManager({
    signinRedirectCallback: async () => user({ expired: false, state: { returnTo: "//evil.example.com" } }),
  });

  await restoreSession(manager);

  expect(window.location.pathname + window.location.search).toBe("/");
});

test("トークンが切れたら、ログインを終わらせて知らせを出し、画面の移動を頼む", async () => {
  const { manager } = fakeManager({ getUser: async () => user({ expired: false }) });
  const { manager: full, handlers } = fakeFullManager(manager);
  const onExpired = vi.fn<() => void>();
  const auth = createAuth(full, config, onExpired);
  const listener = vi.fn<() => void>();
  auth.subscribe(listener);
  expect(await auth.restore()).not.toBeNull();

  handlers["expired"]?.();

  expect(auth.getSnapshot()).toEqual({ user: null, notice: SESSION_EXPIRED });
  expect(await auth.restore()).toBeNull();
  expect(onExpired).toHaveBeenCalledOnce();
  expect(listener).toHaveBeenCalled();
});

test("戻り先は同じサイトの中のパスだけ(外のサイトへ飛ばさない)", () => {
  expect(safeReturnTo("/payslips?staffId=1")).toBe("/payslips?staffId=1");
  expect(safeReturnTo("https://evil.example.com")).toBeUndefined();
  expect(safeReturnTo("//evil.example.com")).toBeUndefined();
  expect(safeReturnTo("/\\evil.example.com")).toBeUndefined();
  expect(safeReturnTo(42)).toBeUndefined();
});
