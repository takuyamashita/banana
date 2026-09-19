#!/usr/bin/env bash
# worktree ごとに別の依存サービス(compose)とポートで動かすための補助。
#
#   scripts/worktree.sh new <名前>     ../<main のディレクトリ名>-<名前> に worktree を作り、.env.worktree を書く
#   scripts/worktree.sh remove <名前>  その worktree の compose(データも)を消し、worktree を外す(ブランチは残す)
#   scripts/worktree.sh list           worktree ごとのスロットとポート
#
# 各 worktree には 1〜9 のスロットを割り当て、ポートは「既定値 + スロット × 100」にする。
# main の worktree はスロット 0(.env.worktree なし)で、既定値のまま動く。
set -euo pipefail

# 名前 → 既定のポート。compose.yaml・mise.toml・vite.config.ts・playwright.config.ts の既定値とそろえる
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

# worktree のスロット。.env.worktree がなければ 0(main と同じポート)
slot_of() {
  local file="$1/.env.worktree"
  if [[ -f $file ]]; then sed -n 's/^WORKTREE_SLOT=//p' "$file"; else echo 0; fi
}

# ブランチ名の / などはディレクトリ名と compose のプロジェクト名に使えないので - にする
slug() { echo "$1" | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9\n-' '-'; }

dir_of() { echo "$(dirname "$(main_dir)")/$(basename "$(main_dir)")-$(slug "$1")"; }

cmd_new() {
  local name=${1:?"worktree の名前(ブランチ名)を指定してください"}
  local dir; dir=$(dir_of "$name")
  [[ -e $dir ]] && { echo "既にあります: $dir" >&2; exit 1; }

  # 使われていない一番小さいスロットを選ぶ
  local used=" " d
  for d in $(worktree_dirs); do used+="$(slot_of "$d") "; done
  local slot
  for slot in $(seq 1 9); do [[ $used != *" $slot "* ]] && break; done
  [[ $used == *" $slot "* ]] && { echo "空いているスロットがありません(最大 9)" >&2; exit 1; }

  if git show-ref --verify --quiet "refs/heads/$name"; then
    git worktree add "$dir" "$name"
  else
    git worktree add -b "$name" "$dir"
  fi

  local project; project=$(basename "$dir")
  {
    echo "# scripts/worktree.sh が書いた、この worktree 専用のポートと compose のプロジェクト名"
    echo "WORKTREE_SLOT=$slot"
    echo "COMPOSE_NAME=$project"
    local entry
    for entry in "${PORTS[@]}"; do echo "${entry%%=*}=$((${entry#*=} + slot * 100))"; done
  } > "$dir/.env.worktree"

  # .env(ローカル用のダミー認証情報)は git に入らないので main からコピーする
  [[ -f "$(main_dir)/.env" && ! -f "$dir/.env" ]] && cp "$(main_dir)/.env" "$dir/.env"
  mise trust --quiet "$dir"
  # node_modules は worktree ごとに要る(pnpm のストアは共有なので速い)。target/ は初回ビルドで作られる
  (cd "$dir" && pnpm install --frozen-lockfile --silent)

  echo
  echo "worktree: $dir(スロット $slot、compose: $project)"
  grep -E '^(API|WEB|KEYCLOAK|MYSQL)_PORT=' "$dir/.env.worktree" | sed 's/^/  /'
  echo "次は: cd $dir && mise run e2e(依存サービスの起動とマイグレーションも行う)"
}

cmd_remove() {
  local name=${1:?"worktree の名前を指定してください"}
  local dir; dir=$(dir_of "$name")
  [[ -f "$dir/.env.worktree" ]] || { echo "worktree:new で作った worktree ではありません: $dir" >&2; exit 1; }
  local project; project=$(sed -n 's/^COMPOSE_NAME=//p' "$dir/.env.worktree")
  docker compose --project-name "$project" down --volumes --remove-orphans
  git worktree remove "$dir"
  echo "片付けました: $dir(ブランチ $name は残しています。不要なら git branch -d $name)"
}

cmd_list() {
  local d
  printf '%-50s %-5s %-6s %-6s %-6s %s\n' WORKTREE SLOT API WEB MYSQL KEYCLOAK
  for d in $(worktree_dirs); do
    local slot; slot=$(slot_of "$d")
    printf '%-50s %-5s %-6s %-6s %-6s %s\n' "$d" "$slot" \
      $((50051 + slot * 100)) $((5173 + slot * 100)) $((3306 + slot * 100)) $((8080 + slot * 100))
  done
}

case "${1:-}" in
  new) shift; cmd_new "$@" ;;
  remove) shift; cmd_remove "$@" ;;
  list) cmd_list ;;
  *) sed -n '2,9p' "$0"; exit 1 ;;
esac
