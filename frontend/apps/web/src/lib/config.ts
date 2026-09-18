export interface RuntimeConfig {
  apiBaseUrl: string;
  oidc: { authority: string; clientId: string };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isRuntimeConfig(value: unknown): value is RuntimeConfig {
  if (!isRecord(value) || typeof value["apiBaseUrl"] !== "string") return false;
  const oidc = value["oidc"];
  return isRecord(oidc) && typeof oidc["authority"] === "string" && typeof oidc["clientId"] === "string";
}

export async function loadRuntimeConfig(): Promise<RuntimeConfig> {
  const res = await fetch("/config.json", { cache: "no-store" });
  if (!res.ok) throw new Error(`failed to load /config.json: ${res.status}`);
  const body: unknown = await res.json();
  if (!isRuntimeConfig(body)) throw new Error("invalid /config.json");
  return body;
}
