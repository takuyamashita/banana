#!/usr/bin/env bash
# worktree ごとに別の依存サービス(compose)とポートで動かすための補助。
#
#   scripts/worktree.sh new <名前>     ../<main のディレクトリ名>-<名前> に worktree を作り、ポートと接続先を .env に書く
#   scripts/worktree.sh remove <名前>  worktree を外し、その compose(データも)を消す(ブランチは残す)
#   scripts/worktree.sh list           worktree ごとのスロットとポート
#
# 各 worktree には 1〜9 のスロットを割り当て、ポートは「既定値 + スロット × 100」にする。
# main の worktree はスロット 0 で、既定値のまま動く(.env にはダミーの認証情報だけがある)。
# 値は worktree の .env に書く。docker compose はプロジェクトの .env を自分で読み、mise も読むので、
# mise を通さずに docker compose を打っても、その worktree の組を操作する
set -euo pipefail

# 名前 → 既定のポート。compose.yaml・vite.config.ts・playwright.config.ts・config/local.toml の既定値とそろえる
PORTS=(
  API_PORT=50051
  WEB_PORT=5173
  MYSQL_PORT=3306
  KEYCLOAK_PORT=8080
  ELASTICMQ_PORT=9324
  OTLP_GRPC_PORT=4317
  OTLP_HTTP_PORT=4318
  JAEGER_UI_PORT=16686
  S3_PORT=8333
  SEAWEED_FILER_PORT=8888
  SEAWEED_MASTER_PORT=9333
)

main_dir() { git worktree list --porcelain | awk '/^worktree /{print $2; exit}'; }
worktree_dirs() { git worktree list --porcelain | awk '/^worktree /{print $2}'; }

# worktree の .env から値を読む。なければ失敗を返す
env_value() {
  local value=""
  [[ -f "$1/.env" ]] && value=$(sed -n "s/^$2=//p" "$1/.env" | tail -n 1)
  [[ -n $value ]] && echo "$value"
}

# worktree のスロット。worktree:new で作っていなければ 0(main と同じポート)
slot_of() { local slot; slot=$(env_value "$1" WORKTREE_SLOT || true); echo "${slot:-0}"; }

# スロットのポートがどれか使われているか(worktree を手で消して compose だけが残っている場合など)
ports_in_use() {
  local slot=$1 entry port
  for entry in "${PORTS[@]}"; do
    port=$((${entry#*=} + slot * 100))
    ss -ltnH "sport = :$port" 2>/dev/null | grep -q . && return 0
  done
  return 1
}

# ブランチ名の / などはディレクトリ名と compose のプロジェクト名に使えないので - にする
slug() { echo "$1" | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9\n-' '-'; }

dir_of() { echo "$(dirname "$(main_dir)")/$(basename "$(main_dir)")-$(slug "$1")"; }

# 英数字を含まない名前(日本語だけなど)は、どれも同じ「---」になって見分けられない
check_name() {
  [[ $(slug "$1") =~ [a-z0-9] ]] || { echo "名前には英数字を含めてください: $1" >&2; exit 1; }
}

cmd_new() {
  local name=${1:?"worktree の名前(ブランチ名)を指定してください"}
  check_name "$name"
  local dir; dir=$(dir_of "$name")
  # feature/x と feature-x のように、違う名前が同じディレクトリになることもある
  [[ -e $dir ]] && { echo "既にあります: $dir" >&2; exit 1; }

  # 使われていない一番小さいスロットを選ぶ。他の worktree が使っているものと、ポートが塞がっているものは避ける
  local used=" " d slot="" candidate
  for d in $(worktree_dirs); do used+="$(slot_of "$d") "; done
  for candidate in $(seq 1 9); do
    if [[ $used != *" $candidate "* ]] && ! ports_in_use "$candidate"; then slot=$candidate; break; fi
  done
  [[ -n $slot ]] || { echo "空いているスロットがありません(最大 9。docker compose ls で残っている組も確かめてください)" >&2; exit 1; }

  if git show-ref --verify --quiet "refs/heads/$name"; then
    git worktree add "$dir" "$name"
  elif git show-ref --verify --quiet "refs/remotes/origin/$name"; then
    # リモートにしかないブランチ(他の人の PR など)は、それを追跡するブランチとして取り出す
    git worktree add --track -b "$name" "$dir" "origin/$name"
  else
    # 新しいブランチは、今いる worktree の HEAD から作る
    git worktree add -b "$name" "$dir"
  fi

  # .env(ローカル用のダミー認証情報)は git に入らないので main からコピーし、この worktree の値を足す
  if [[ -f "$(main_dir)/.env" ]]; then cp "$(main_dir)/.env" "$dir/.env"; fi
  local project; project=$(basename "$dir")
  local -A port
  local entry
  for entry in "${PORTS[@]}"; do port[${entry%%=*}]=$((${entry#*=} + slot * 100)); done
  {
    echo
    echo "# ---- scripts/worktree.sh が書いた、この worktree 専用の値(スロット $slot) ----"
    echo "WORKTREE_SLOT=$slot"
    echo "COMPOSE_NAME=$project"
    for entry in "${PORTS[@]}"; do echo "${entry%%=*}=${port[${entry%%=*}]}"; done
    echo "DATABASE_URL=mysql://platform:platform@127.0.0.1:${port[MYSQL_PORT]}/platform"
    # server・migrate・Lambda の接続先(config/local.toml の値を上書きする)
    echo "APP__DATABASE__URL=mysql://platform:platform@127.0.0.1:${port[MYSQL_PORT]}/platform"
    echo "APP__SERVER__ADDR=0.0.0.0:${port[API_PORT]}"
    echo "APP__SERVER__CORS_ALLOWED_ORIGINS=http://localhost:${port[WEB_PORT]}"
    echo "APP__AUTH__ISSUER=http://localhost:${port[KEYCLOAK_PORT]}/realms/platform"
    echo "APP__AUTH__KEYCLOAK_BASE_URL=http://localhost:${port[KEYCLOAK_PORT]}"
    echo "APP__MESSAGING__QUEUE_URL=http://localhost:${port[ELASTICMQ_PORT]}/000000000000/payroll-events.fifo"
    echo "APP__MESSAGING__SQS_ENDPOINT=http://localhost:${port[ELASTICMQ_PORT]}"
    echo "APP__TELEMETRY__OTLP_ENDPOINT=http://localhost:${port[OTLP_GRPC_PORT]}"
  } >> "$dir/.env"

  mise trust --quiet "$dir"
  # node_modules は worktree ごとに要る(pnpm のストアは共有なので速い)。target/ は初回ビルドで作られる
  (cd "$dir" && pnpm install --frozen-lockfile --silent)

  echo
  echo "worktree: $dir(スロット $slot、compose: $project)"
  grep -E '^(API|WEB|KEYCLOAK|MYSQL)_PORT=' "$dir/.env" | sed 's/^/  /'
  echo "次は: cd $dir && mise run e2e(依存サービスの起動とマイグレーションも行う)"
}

cmd_remove() {
  local name=${1:?"worktree の名前を指定してください"}
  check_name "$name"
  local dir; dir=$(dir_of "$name")
  local project; project=$(env_value "$dir" COMPOSE_NAME || true)
  [[ -n $project ]] || { echo "worktree:new で作った worktree ではありません: $dir" >&2; exit 1; }

  # 名前が同じディレクトリに潰れる別の worktree(feature/x と feature-x)を消さないよう、ブランチを確かめる
  local branch; branch=$(git -C "$dir" branch --show-current)
  [[ $branch == "$name" ]] || { echo "$dir はブランチ $branch の worktree です($name ではありません)" >&2; exit 1; }
  # データを消す前に、コミットしていない変更がないことを確かめる
  if [[ -n $(git -C "$dir" status --porcelain) ]]; then
    echo "コミットしていない変更があります。コミットするか片付けてから実行してください: $dir" >&2
    git -C "$dir" status --short >&2
    exit 1
  fi

  git worktree remove "$dir"
  # compose ファイルがなくても、プロジェクト名だけで片付けられる
  docker compose --project-name "$project" down --volumes --remove-orphans
  echo "片付けました: $dir(ブランチ $name は残しています。不要なら git branch -d $name)"
}

cmd_list() {
  local d
  printf '%-50s %-5s %-6s %-6s %-6s %s\n' WORKTREE SLOT API WEB MYSQL KEYCLOAK
  for d in $(worktree_dirs); do
    local slot; slot=$(slot_of "$d")
    # main(スロット 0)は .env にポートを持たないので、既定値を出す
    printf '%-50s %-5s %-6s %-6s %-6s %s\n' "$d" "$slot" \
      "$(env_value "$d" API_PORT || echo 50051)" "$(env_value "$d" WEB_PORT || echo 5173)" \
      "$(env_value "$d" MYSQL_PORT || echo 3306)" "$(env_value "$d" KEYCLOAK_PORT || echo 8080)"
  done
}

case "${1:-}" in
  new) shift; cmd_new "$@" ;;
  remove) shift; cmd_remove "$@" ;;
  list) cmd_list ;;
  *) sed -n '2,11p' "$0"; exit 1 ;;
esac
