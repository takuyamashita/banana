// proto から buf generate した型とサービス記述子(src/gen)を、使いやすい形で公開する
import type { DescService } from "@bufbuild/protobuf";
import { createClient, type Client, type Interceptor, type Transport } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";

import { PayrollService } from "./gen/acme/payroll/v1/payroll_pb";
import { ProjectService } from "./gen/acme/payroll/v1/project_pb";
import { StaffService } from "./gen/acme/payroll/v1/staff_pb";
import { TimesheetService } from "./gen/acme/timesheet/v1/timesheet_pb";

export * from "./gen/acme/payroll/v1/payroll_pb";
export * from "./gen/acme/payroll/v1/project_pb";
export * from "./gen/acme/payroll/v1/staff_pb";
// 勤怠は案件(Project)などの名前が給与と重なるので、名前空間に分けて出す
export * as timesheet from "./gen/acme/timesheet/v1/timesheet_pb";
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

/// サービスごとに宛先を分ける transport。`routes` にないサービスは `fallback` に送る。
/// 画面はサービスがいくつあっても1つの transport だけを扱い、どのサーバーに届くかを知らない
export function routeByService(routes: ReadonlyMap<DescService, Transport>, fallback: Transport): Transport {
  const pick = (service: DescService) => routes.get(service) ?? fallback;
  return {
    unary: (method, ...rest) => pick(method.parent).unary(method, ...rest),
    stream: (method, ...rest) => pick(method.parent).stream(method, ...rest),
  };
}

export interface ServiceUrls {
  /// 給与(payroll)サービス
  apiBaseUrl: string;
  /// 勤怠(timesheet)サービス。AWS では給与と同じ ALB(パスで振り分ける)なので同じ URL になる
  timesheetApiBaseUrl: string;
}

/// サービスごとの宛先に振り分ける transport。画面は connect-query にこの transport を渡す
export function createServicesTransport(urls: ServiceUrls, getAccessToken: ApiOptions["getAccessToken"]): Transport {
  const payroll = createApiTransport({ baseUrl: urls.apiBaseUrl, getAccessToken });
  const timesheet = createApiTransport({ baseUrl: urls.timesheetApiBaseUrl, getAccessToken });
  return routeByService(new Map([[TimesheetService, timesheet]]), payroll);
}

export interface ApiClients {
  payroll: Client<typeof PayrollService>;
  staff: Client<typeof StaffService>;
  project: Client<typeof ProjectService>;
  timesheet: Client<typeof TimesheetService>;
}

/// 画面を通さずに API を呼ぶクライアント(E2E のテストデータ投入など)
export function createApiClients(urls: ServiceUrls, getAccessToken: ApiOptions["getAccessToken"]): ApiClients {
  const transport = createServicesTransport(urls, getAccessToken);
  return {
    payroll: createClient(PayrollService, transport),
    staff: createClient(StaffService, transport),
    project: createClient(ProjectService, transport),
    timesheet: createClient(TimesheetService, transport),
  };
}
