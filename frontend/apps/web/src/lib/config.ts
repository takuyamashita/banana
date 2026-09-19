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

export async function loadRuntimeConfig(): Promise<RuntimeConfig> {
  const res = await fetch("/config.json", { cache: "no-store" });
  if (!res.ok) throw new Error(`failed to load /config.json: ${res.status}`);
  const body: unknown = await res.json();
  if (!isRuntimeConfig(body)) throw new Error("invalid /config.json");
  return body;
}
