import { loadRuntimeConfig } from "./config";

const respond = (body: string, contentType: string, status = 200) =>
  vi.fn<typeof fetch>(async () => new Response(body, { status, headers: { "content-type": contentType } }));

test("正しい設定を読む", async () => {
  const body = JSON.stringify({
    apiBaseUrl: "https://api.example.com",
    timesheetApiBaseUrl: "https://api.example.com",
    oidc: { authority: "https://id", clientId: "web" },
  });
  await expect(loadRuntimeConfig(respond(body, "application/json"))).resolves.toMatchObject({
    apiBaseUrl: "https://api.example.com",
    timesheetApiBaseUrl: "https://api.example.com",
  });
});

test("置き忘れて index.html が返ってきたら、設定ファイルが見つからないと伝える", async () => {
  await expect(loadRuntimeConfig(respond("<!doctype html>", "text/html"))).rejects.toThrow(
    "設定ファイル(/config.json)が見つかりません(HTTP 200)。",
  );
});

test("形の違う設定は受け付けない", async () => {
  await expect(loadRuntimeConfig(respond(JSON.stringify({ apiBaseUrl: 1 }), "application/json"))).rejects.toThrow(
    "設定ファイル(/config.json)の形式が正しくありません。",
  );
});
