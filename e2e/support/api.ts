// テストデータは DB に直接 INSERT せず、api-client 経由で API から投入する。
// 値にはランダムな接尾辞を付け、並列実行で衝突しないようにする
import { createApiClients, type ApiClients } from "@platform/api-client";

const API = process.env["E2E_API_URL"] ?? "http://localhost:50051";
const KEYCLOAK = process.env["E2E_KEYCLOAK_URL"] ?? "http://localhost:8080";

export const ADMIN = { email: "admin@example.com", password: "password" };

export function unique(prefix: string): string {
  return `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
}

async function passwordGrant(email: string, password: string): Promise<string> {
  const res = await fetch(`${KEYCLOAK}/realms/platform/protocol/openid-connect/token`, {
    method: "POST",
    body: new URLSearchParams({ grant_type: "password", client_id: "web", username: email, password }),
  });
  if (!res.ok) throw new Error(`token request failed: ${res.status} ${await res.text()}`);
  const body: unknown = await res.json();
  if (typeof body !== "object" || body === null || !("access_token" in body) || typeof body.access_token !== "string") {
    throw new Error("access_token missing");
  }
  return body.access_token;
}

export async function adminApi(): Promise<ApiClients> {
  const token = await passwordGrant(ADMIN.email, ADMIN.password);
  return createApiClients({ baseUrl: API, getAccessToken: async () => token });
}

export interface SeededStaff {
  staffId: bigint;
  email: string;
  displayName: string;
  temporaryPassword: string;
}

export async function seedStaff(api: ApiClients): Promise<SeededStaff> {
  const email = `${unique("staff")}@example.com`;
  const displayName = unique("派遣");
  const temporaryPassword = "Temp-pass-1";
  const { staffId } = await api.staff.createStaff({ email, displayName, temporaryPassword });
  return { staffId, email, displayName, temporaryPassword };
}

export async function seedProject(api: ApiClients): Promise<{ projectId: bigint; name: string }> {
  const name = unique("案件");
  const { projectId } = await api.project.createProject({ name });
  return { projectId, name };
}
