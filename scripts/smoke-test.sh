#!/usr/bin/env bash
# 起動中のローカル server に対して、主要なシナリオを grpcurl で一通り流す。
# 前提: docker compose の依存サービスと server(mise run dev-backend)が起動済み
set -euo pipefail

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

call() { # call <token> <method> <json> → 標準出力に結果(エラー時はコード)
  g -H "authorization: Bearer $1" -d "$3" "$API" "acme.payroll.v1.$2" 2>&1 || true
}

check "health" "ok" "$(curl -s "http://$API/health")"
check "未認証は Unauthenticated" "Unauthenticated" "$(g -d '{}' "$API" acme.payroll.v1.ProjectService/ListProjects 2>&1 || true)"

ADMIN=$(token admin@example.com)
PROJECT_ID=$(call "$ADMIN" ProjectService/CreateProject "{\"name\":\"案件-$SUFFIX\"}" | jq -r .projectId)
check "案件登録" "" "$PROJECT_ID"

TARO=taro-$SUFFIX@example.com
HANAKO=hanako-$SUFFIX@example.com
TARO_ID=$(call "$ADMIN" StaffService/CreateStaff "{\"email\":\"$TARO\",\"display_name\":\"派遣 太郎\",\"temporary_password\":\"Temp-pass-1\"}" | jq -r .staffId)
HANAKO_ID=$(call "$ADMIN" StaffService/CreateStaff "{\"email\":\"$HANAKO\",\"display_name\":\"派遣 花子\",\"temporary_password\":\"Temp-pass-1\"}" | jq -r .staffId)
check "派遣社員登録" "" "$TARO_ID/$HANAKO_ID"
check "同じメールの再登録は AlreadyExists" "AlreadyExists" \
  "$(call "$ADMIN" StaffService/CreateStaff "{\"email\":\"$TARO\",\"display_name\":\"x\",\"temporary_password\":\"Temp-pass-1\"}")"

LINES="[{\"project_id\":$PROJECT_ID,\"work_minutes\":9600,\"hourly_rate\":1501},{\"project_id\":$PROJECT_ID,\"work_minutes\":100,\"hourly_rate\":1500}]"
CREATE="{\"staff_id\":$TARO_ID,\"pay_year\":2026,\"pay_month\":9,\"lines\":$LINES}"
PAYSLIP_ID=$(call "$ADMIN" PayrollService/CreatePayslip "$CREATE" | jq -r .payslipId)
check "給与明細の作成" "" "$PAYSLIP_ID"
check "作成直後は作成中" "PAYSLIP_STATUS_DRAFT" \
  "$(call "$ADMIN" PayrollService/GetPayslip "{\"payslip_id\":$PAYSLIP_ID}")"
check "同じ月の作成は AlreadyExists" "AlreadyExists" "$(call "$ADMIN" PayrollService/CreatePayslip "$CREATE")"
check "月=257 は InvalidArgument(as キャストなら1月になる)" "InvalidArgument" \
  "$(call "$ADMIN" PayrollService/CreatePayslip "{\"staff_id\":$TARO_ID,\"pay_year\":2026,\"pay_month\":257,\"lines\":$LINES}")"
check "14分の稼働は InvalidArgument" "InvalidArgument" \
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
DRAFT_ID=$(call "$ADMIN" PayrollService/CreatePayslip "{\"staff_id\":$TARO_ID,\"pay_year\":2026,\"pay_month\":10,\"lines\":$LINES}" | jq -r .payslipId)

# 9600分×1501円/60 = 240,160円、100分→90分×1500円/60 = 2,250円
check "合計は円未満切り捨て・15分単位" '"totalYen": "242410"' \
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

echo
echo "passed: $PASS, failed: $FAIL"
[[ $FAIL -eq 0 ]]
