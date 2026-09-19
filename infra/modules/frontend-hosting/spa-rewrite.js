// 画面の URL(/payslips など、拡張子のないパス)を index.html に置き換える。画面の切り替えは
// ブラウザの中のルーターが行うので、どの画面の URL で開いても(再読み込み・ブックマーク)同じ HTML を返す。
// 拡張子のあるパス(assets/・config.json)は置き換えない。無ければ 404 のままにし、置き忘れを HTML で隠さない
// oxlint-disable-next-line no-unused-vars -- CloudFront が呼ぶ入口(このファイルの中からは呼ばない)
function handler(event) {
  var request = event.request;
  var last = request.uri.split("/").pop();
  if (last.indexOf(".") === -1) request.uri = "/index.html";
  return request;
}
