export interface RuntimeConfig {
  apiBaseUrl: string;
  oidc: {
    authority: string;
    clientId: string;
    /// ディスカバリに載っていない宛先を補う(Cognito の /logout・/oauth2/revoke)。Keycloak では不要
    endSessionEndpoint?: string | null;
    revocationEndpoint?: string | null;
    /// ログアウト後の戻り先を渡す引数名。OIDC の標準(post_logout_redirect_uri)と違う IdP だけ指定する(Cognito は logout_uri)
    postLogoutRedirectParam?: string | null;
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isRuntimeConfig(value: unknown): value is RuntimeConfig {
  if (!isRecord(value) || typeof value["apiBaseUrl"] !== "string") return false;
  const oidc = value["oidc"];
  // 指定しない項目は、無いか null(Terraform の optional は null で書き出す)
  const optionalString = (v: unknown) => v === undefined || v === null || typeof v === "string";
  return (
    isRecord(oidc) &&
    typeof oidc["authority"] === "string" &&
    typeof oidc["clientId"] === "string" &&
    optionalString(oidc["endSessionEndpoint"]) &&
    optionalString(oidc["revocationEndpoint"]) &&
    optionalString(oidc["postLogoutRedirectParam"])
  );
}

/// 実行時の設定(/config.json)を読む。読めなければ、画面に出せる文言の Error を投げる
export async function loadRuntimeConfig(fetcher: typeof fetch = fetch): Promise<RuntimeConfig> {
  const res = await fetcher("/config.json", { cache: "no-store" });
  // 置き忘れると、配信の設定によっては index.html が 200 で返る。JSON でなければ置かれていないとみなす
  if (!res.ok || !res.headers.get("content-type")?.includes("application/json")) {
    throw new Error(`設定ファイル(/config.json)が見つかりません(HTTP ${res.status})。`);
  }
  const body: unknown = await res.json().catch(() => undefined);
  if (!isRuntimeConfig(body)) throw new Error("設定ファイル(/config.json)の形式が正しくありません。");
  return body;
}
