#!/usr/bin/env bash
# 起動中のローカル server に対して、主要なシナリオを grpcurl で一通り流す。
# 給与と勤怠のサービスの間の出来事(派遣社員・案件の登録 → 勤怠、勤務表の承認 → 給与)も確かめる。
# 前提: docker compose の依存サービスと server(給与・勤怠。mise run dev-backend)が起動済み
set -euo pipefail
# proto の import パスがリポジトリ直下からの相対なので、どこから実行してもリポジトリ直下で動かす
cd "$(dirname "$0")/.."

API=${API:-localhost:${API_PORT:-50051}}
KEYCLOAK=${KEYCLOAK:-http://localhost:${KEYCLOAK_PORT:-8080}}
SUFFIX=$(date +%s%N | tail -c 7)
PASS=0
FAIL=0

g() {
  grpcurl -plaintext -import-path proto \
    -proto acme/payroll/v1/payroll.proto -proto acme/payroll/v1/staff.proto -proto acme/payroll/v1/project.proto \
    "$@"
}

token() {
  curl -sf -d grant_type=password -d client_id=web -d "username=$1" -d "password=${2:-password}" \
    "$KEYCLOAK/realms/platform/protocol/openid-connect/token" | jq -r .access_token
}

# 管理APIで仮パスワードを確定させる(ブラウザなら初回ログインでパスワード変更を求められる)
confirm_password() {
  local kc uid
  kc=$(curl -sf -d grant_type=password -d client_id=admin-cli -d username=admin -d password=admin \
    "$KEYCLOAK/realms/master/protocol/openid-connect/token" | jq -r .access_token)
  uid=$(curl -sf -H "Authorization: Bearer $kc" "$KEYCLOAK/admin/realms/platform/users?email=$1&exact=true" | jq -r '.[0].id')
  curl -sf -X PUT -H "Authorization: Bearer $kc" -H 'content-type: application/json' \
    -d '{"type":"password","value":"password","temporary":false}' \
    "$KEYCLOAK/admin/realms/platform/users/$uid/reset-password"
}

check() {
  local name=$1 expected=$2 actual=$3
  if [[ "$actual" == *"$expected"* ]]; then
    PASS=$((PASS + 1)); echo "ok   $name"
  else
    FAIL=$((FAIL + 1)); echo "FAIL $name"; echo "     expected: $expected"; echo "     actual:   $actual"
  fi
}

# 番号が返ったか。作成に失敗すると空や null になるので、数字であることを確かめる
check_id() {
  local name=$1; shift
  local id
  for id in "$@"; do
    if [[ ! $id =~ ^[0-9]+$ ]]; then
      FAIL=$((FAIL + 1)); echo "FAIL $name"; echo "     番号が返らなかった: '$id'"; return
    fi
  done
  PASS=$((PASS + 1)); echo "ok   $name"
}

# 応答から番号を取り出す。失敗した応答(JSON でない)なら空にし、check_id で FAIL にする
id_of() { jq -r "$1 // empty" 2>/dev/null || true; }

call() { # call <token> <method> <json> → 標準出力に結果(エラー時はコード)
  g -H "authorization: Bearer $1" -d "$3" "$API" "acme.payroll.v1.$2" 2>&1 || true
}

check "health" "ok" "$(curl -s "http://$API/health")"
check "ready(DB にも届く)" "ok" "$(curl -s "http://$API/ready")"
check "未認証は Unauthenticated" "Unauthenticated" "$(g -d '{}' "$API" acme.payroll.v1.ProjectService/ListProjects 2>&1 || true)"

ADMIN=$(token admin@example.com)
PROJECT_ID=$(call "$ADMIN" ProjectService/CreateProject "{\"name\":\"案件-$SUFFIX\"}" | id_of .projectId)
check_id "案件登録" "$PROJECT_ID"

TARO=taro-$SUFFIX@example.com
HANAKO=hanako-$SUFFIX@example.com
TARO_ID=$(call "$ADMIN" StaffService/CreateStaff "{\"email\":\"$TARO\",\"display_name\":\"派遣 太郎\",\"temporary_password\":\"Temp-pass-1\"}" | id_of .staffId)
HANAKO_ID=$(call "$ADMIN" StaffService/CreateStaff "{\"email\":\"$HANAKO\",\"display_name\":\"派遣 花子\",\"temporary_password\":\"Temp-pass-1\"}" | id_of .staffId)
check_id "派遣社員登録" "$TARO_ID" "$HANAKO_ID"
check "同じメールの再登録は AlreadyExists" "AlreadyExists" \
  "$(call "$ADMIN" StaffService/CreateStaff "{\"email\":\"$TARO\",\"display_name\":\"x\",\"temporary_password\":\"Temp-pass-1\"}")"

LINES="[{\"project_id\":$PROJECT_ID,\"work_minutes\":9600,\"hourly_rate\":1501},{\"project_id\":$PROJECT_ID,\"work_minutes\":90,\"hourly_rate\":1500}]"
CREATE="{\"staff_id\":$TARO_ID,\"pay_year\":2026,\"pay_month\":9,\"lines\":$LINES}"
PAYSLIP_ID=$(call "$ADMIN" PayrollService/CreatePayslip "$CREATE" | id_of .payslipId)
check_id "給与明細の作成" "$PAYSLIP_ID"
check "作成直後は作成中" "PAYSLIP_STATUS_DRAFT" \
  "$(call "$ADMIN" PayrollService/GetPayslip "{\"payslip_id\":$PAYSLIP_ID}")"
check "明細行に作成時点の案件名が残る" "\"projectName\": \"案件-$SUFFIX\"" \
  "$(call "$ADMIN" PayrollService/GetPayslip "{\"payslip_id\":$PAYSLIP_ID}")"
check "同じ月の作成は AlreadyExists" "AlreadyExists" "$(call "$ADMIN" PayrollService/CreatePayslip "$CREATE")"
check "月=257 は InvalidArgument(as キャストなら1月になる)" "InvalidArgument" \
  "$(call "$ADMIN" PayrollService/CreatePayslip "{\"staff_id\":$TARO_ID,\"pay_year\":2026,\"pay_month\":257,\"lines\":$LINES}")"
check "15分単位でない稼働(14分)は InvalidArgument" "InvalidArgument" \
  "$(call "$ADMIN" PayrollService/CreatePayslip "{\"staff_id\":$TARO_ID,\"pay_year\":2026,\"pay_month\":10,\"lines\":[{\"project_id\":$PROJECT_ID,\"work_minutes\":14,\"hourly_rate\":1000}]}")"
check "存在しない派遣社員は InvalidArgument" "InvalidArgument" \
  "$(call "$ADMIN" PayrollService/CreatePayslip "{\"staff_id\":999999,\"pay_year\":2026,\"pay_month\":9,\"lines\":$LINES}")"
check "存在しない案件は InvalidArgument" "InvalidArgument" \
  "$(call "$ADMIN" PayrollService/CreatePayslip "{\"staff_id\":$TARO_ID,\"pay_year\":2026,\"pay_month\":11,\"lines\":[{\"project_id\":999999,\"work_minutes\":600,\"hourly_rate\":1000}]}")"

call "$ADMIN" PayrollService/FinalizePayslip "{\"payslip_id\":$PAYSLIP_ID}" >/dev/null
check "確定すると確定済み" "PAYSLIP_STATUS_FINALIZED" \
  "$(call "$ADMIN" PayrollService/GetPayslip "{\"payslip_id\":$PAYSLIP_ID}")"
check "二重確定は FailedPrecondition" "FailedPrecondition" \
  "$(call "$ADMIN" PayrollService/FinalizePayslip "{\"payslip_id\":$PAYSLIP_ID}")"
check "存在しない給与明細の確定は NotFound" "NotFound" \
  "$(call "$ADMIN" PayrollService/FinalizePayslip "{\"payslip_id\":999999}")"
# 本人に見えないことを確かめるための、作成中のままの10月分
DRAFT_ID=$(call "$ADMIN" PayrollService/CreatePayslip "{\"staff_id\":$TARO_ID,\"pay_year\":2026,\"pay_month\":10,\"lines\":$LINES}" | id_of .payslipId)

# 9600分×1501円/60 = 240,160円、90分×1500円/60 = 2,250円
check "合計は円未満切り捨て" '"totalYen": "242410"' \
  "$(call "$ADMIN" PayrollService/GetPayslip "{\"payslip_id\":$PAYSLIP_ID}")"

confirm_password "$TARO"
confirm_password "$HANAKO"
TARO_TOKEN=$(token "$TARO")
HANAKO_TOKEN=$(token "$HANAKO")

check "本人は GetMe で自分の staff_id が分かる" "\"staffId\": \"$TARO_ID\"" "$(call "$TARO_TOKEN" StaffService/GetMe '{}')"
check "本人は自分の明細を見られる" "\"payslipId\": \"$PAYSLIP_ID\"" \
  "$(call "$TARO_TOKEN" PayrollService/GetPayslip "{\"payslip_id\":$PAYSLIP_ID}")"
check "他人の明細は NotFound" "NotFound" \
  "$(call "$HANAKO_TOKEN" PayrollService/GetPayslip "{\"payslip_id\":$PAYSLIP_ID}")"
check "他人の一覧は NotFound" "NotFound" \
  "$(call "$HANAKO_TOKEN" PayrollService/ListPayslips "{\"staff_id\":$TARO_ID}")"
check "本人にも作成中の明細は NotFound" "NotFound" \
  "$(call "$TARO_TOKEN" PayrollService/GetPayslip "{\"payslip_id\":$DRAFT_ID}")"
TARO_LIST=$(call "$TARO_TOKEN" PayrollService/ListPayslips "{\"staff_id\":$TARO_ID}" | jq -r '[.payslips[].payslipId] | join(",")')
check "本人の一覧は確定済みだけ(作成中の明細は出ない)" "ids=$PAYSLIP_ID." "ids=$TARO_LIST."
check "派遣社員は給与明細を作成できない" "PermissionDenied" \
  "$(call "$TARO_TOKEN" PayrollService/CreatePayslip "{\"staff_id\":$TARO_ID,\"pay_year\":2026,\"pay_month\":11,\"lines\":$LINES}")"
check "派遣社員は給与明細を確定できない" "PermissionDenied" \
  "$(call "$TARO_TOKEN" PayrollService/FinalizePayslip "{\"payslip_id\":$DRAFT_ID}")"

# ---- 勤怠(timesheet)サービス ----
# 給与で登録した派遣社員・案件が出来事で届き、承認した勤務表の稼働が出来事で給与に届く
TIMESHEET_API=${TIMESHEET_API:-localhost:${TIMESHEET_API_PORT:-50052}}
t() { grpcurl -plaintext -import-path proto -proto acme/timesheet/v1/timesheet.proto "$@"; }
tcall() { # tcall <token> <method> <json>
  t -H "authorization: Bearer $1" -d "$3" "$TIMESHEET_API" "acme.timesheet.v1.TimesheetService/$2" 2>&1 || true
}
# 出来事は非同期で届くので、期待した結果になるまで待つ(最大 15 秒)
eventually() {
  local expected=$1 out=""
  shift
  for _ in $(seq 1 30); do
    out=$("$@")
    [[ $out == *"$expected"* ]] && break
    sleep 0.5
  done
  echo "$out"
}

check "勤怠: health" "ok" "$(curl -s "http://$TIMESHEET_API/health")"
check "勤怠: 給与で登録した派遣社員が届く" "派遣 太郎" \
  "$(eventually "派遣 太郎" tcall "$TARO_TOKEN" GetMyTimesheet '{"year":2026,"month":9}')"
check "勤怠: 給与で登録した案件が届く" "案件-$SUFFIX" \
  "$(eventually "案件-$SUFFIX" tcall "$TARO_TOKEN" ListProjects '{}')"
ENTRIES="[{\"date\":\"2026-09-01\",\"project_id\":$PROJECT_ID,\"work_minutes\":480},{\"date\":\"2026-09-02\",\"project_id\":$PROJECT_ID,\"work_minutes\":450}]"
check "勤怠: 本人が稼働を書く" '"totalMinutes": 930' \
  "$(tcall "$TARO_TOKEN" SaveMyTimesheet "{\"year\":2026,\"month\":9,\"entries\":$ENTRIES}")"
check "勤怠: 15分単位でない稼働は InvalidArgument" "InvalidArgument" \
  "$(tcall "$TARO_TOKEN" SaveMyTimesheet "{\"year\":2026,\"month\":10,\"entries\":[{\"date\":\"2026-10-01\",\"project_id\":$PROJECT_ID,\"work_minutes\":470}]}")"
TIMESHEET_ID=$(tcall "$TARO_TOKEN" SubmitMyTimesheet '{"year":2026,"month":9}' | id_of .timesheet.timesheetId)
check_id "勤怠: 申告" "$TIMESHEET_ID"
check "勤怠: 申告した勤務表は書き直せない" "FailedPrecondition" \
  "$(tcall "$TARO_TOKEN" SaveMyTimesheet "{\"year\":2026,\"month\":9,\"entries\":$ENTRIES}")"
check "勤怠: 派遣社員は承認できない" "PermissionDenied" \
  "$(tcall "$TARO_TOKEN" ApproveTimesheet "{\"timesheet_id\":$TIMESHEET_ID}")"
check "勤怠: 管理者が承認する" "TIMESHEET_STATUS_APPROVED" \
  "$(tcall "$ADMIN" ApproveTimesheet "{\"timesheet_id\":$TIMESHEET_ID}")"
check "給与: 承認した稼働が届く(案件ごとの合計)" '"workMinutes": 930' \
  "$(eventually '"workMinutes": 930' call "$ADMIN" PayrollService/GetApprovedWork "{\"staff_id\":$TARO_ID,\"pay_year\":2026,\"pay_month\":9}")"
check "給与: 承認した稼働は派遣社員には見えない" "PermissionDenied" \
  "$(call "$TARO_TOKEN" PayrollService/GetApprovedWork "{\"staff_id\":$TARO_ID,\"pay_year\":2026,\"pay_month\":9}")"

echo
echo "passed: $PASS, failed: $FAIL"
[[ $FAIL -eq 0 ]]
