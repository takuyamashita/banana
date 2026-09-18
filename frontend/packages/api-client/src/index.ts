// proto から buf generate した型とサービス記述子(src/gen)を、使いやすい形で公開する
import { createClient, type Client, type Interceptor } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";

import { PayrollService } from "./gen/acme/payroll/v1/payroll_pb";
import { ProjectService } from "./gen/acme/payroll/v1/project_pb";
import { StaffService } from "./gen/acme/payroll/v1/staff_pb";

export * from "./gen/acme/payroll/v1/payroll_pb";
export * from "./gen/acme/payroll/v1/project_pb";
export * from "./gen/acme/payroll/v1/staff_pb";
export { Code, ConnectError } from "@connectrpc/connect";

export interface ApiClients {
  payroll: Client<typeof PayrollService>;
  staff: Client<typeof StaffService>;
  project: Client<typeof ProjectService>;
}

export interface ApiOptions {
  baseUrl: string;
  /// リクエストごとにアクセストークンを返す。未ログインなら undefined
  getAccessToken: () => Promise<string | undefined>;
  /// fetch を差し替える(テストで MSW を使うときなど)
  fetch?: typeof globalThis.fetch;
}

/// サーバー(tonic + tonic-web)とは gRPC-Web で話す
export function createApiClients(options: ApiOptions): ApiClients {
  const auth: Interceptor = (next) => async (req) => {
    const token = await options.getAccessToken();
    if (token) {
      req.header.set("authorization", `Bearer ${token}`);
    }
    return next(req);
  };
  const transport = createGrpcWebTransport({
    baseUrl: options.baseUrl,
    interceptors: [auth],
    fetch: options.fetch,
  });
  return {
    payroll: createClient(PayrollService, transport),
    staff: createClient(StaffService, transport),
    project: createClient(ProjectService, transport),
  };
}
