// proto から buf generate した型とサービス記述子(src/gen)を、使いやすい形で公開する
import { createClient, type Client, type Interceptor, type Transport } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";

import { PayrollService } from "./gen/acme/payroll/v1/payroll_pb";
import { ProjectService } from "./gen/acme/payroll/v1/project_pb";
import { StaffService } from "./gen/acme/payroll/v1/staff_pb";
import { UserService } from "./gen/acme/payroll/v1/user_pb";

export * from "./gen/acme/payroll/v1/payroll_pb";
export * from "./gen/acme/payroll/v1/project_pb";
export * from "./gen/acme/payroll/v1/staff_pb";
export * from "./gen/acme/payroll/v1/user_pb";
export { Code, ConnectError, type Transport } from "@connectrpc/connect";

export interface ApiOptions {
  baseUrl: string;
  /// リクエストごとにアクセストークンを返す。未ログインなら undefined
  getAccessToken: () => Promise<string | undefined>;
}

/// サーバー(tonic + tonic-web)とは gRPC-Web で話す。画面は connect-query にこの transport を渡す
export function createApiTransport(options: ApiOptions): Transport {
  const auth: Interceptor = (next) => async (req) => {
    const token = await options.getAccessToken();
    if (token) {
      req.header.set("authorization", `Bearer ${token}`);
    }
    return next(req);
  };
  return createGrpcWebTransport({ baseUrl: options.baseUrl, interceptors: [auth] });
}

export interface ApiClients {
  payroll: Client<typeof PayrollService>;
  staff: Client<typeof StaffService>;
  project: Client<typeof ProjectService>;
  user: Client<typeof UserService>;
}

/// 画面を通さずに API を呼ぶクライアント(E2E のテストデータ投入など)
export function createApiClients(options: ApiOptions): ApiClients {
  const transport = createApiTransport(options);
  return {
    payroll: createClient(PayrollService, transport),
    staff: createClient(StaffService, transport),
    project: createClient(ProjectService, transport),
    user: createClient(UserService, transport),
  };
}
