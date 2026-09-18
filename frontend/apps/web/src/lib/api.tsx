import type { ApiClients } from "@platform/api-client";
import { createContext, useContext, type ReactNode } from "react";

const ApiContext = createContext<ApiClients | null>(null);

export function ApiProvider({ clients, children }: { clients: ApiClients; children: ReactNode }) {
  return <ApiContext value={clients}>{children}</ApiContext>;
}

export function useApi(): ApiClients {
  const clients = useContext(ApiContext);
  if (!clients) throw new Error("ApiProvider is missing");
  return clients;
}

/// ConnectError をそのまま画面に出せる文言にする
export function errorMessage(err: unknown): string {
  if (err && typeof err === "object" && "rawMessage" in err && typeof err.rawMessage === "string") {
    return err.rawMessage;
  }
  return err instanceof Error ? err.message : String(err);
}
