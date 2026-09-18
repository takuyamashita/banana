#!/usr/bin/env bash
# レイヤー間の依存ルールを cargo metadata で検査する。違反があれば非0で終了する。
# dev-dependencies(テスト用)は対象外。見るのは通常の依存(normal)だけ
set -euo pipefail
cd "$(dirname "$0")/.."

META=$(cargo metadata --format-version 1 --locked)

# name → 直接の通常依存(workspace 外も含む)
direct() {
  jq -r --arg name "$1" '
    (.packages | map({key: .id, value: .name}) | from_entries) as $names
    | .resolve.nodes[]
    | select($names[.id] == $name)
    | .deps[]
    | select(any(.dep_kinds[]; .kind == null))
    | $names[.pkg]' <<<"$META" | sort -u
}

# name → 推移的な通常依存
transitive() {
  jq -r --arg name "$1" '
    (.packages | map({key: .id, value: .name}) | from_entries) as $names
    | (.resolve.nodes | map({key: .id, value: [.deps[] | select(any(.dep_kinds[]; .kind == null)) | .pkg]}) | from_entries) as $graph
    | (.resolve.nodes[] | select($names[.id] == $name) | .id) as $root
    | [$root] | until(
        . as $seen | ([.[] | $graph[.][]] | unique) - $seen | length == 0;
        . + ([.[] | $graph[.][]] | unique) | unique
      )
    | .[] | $names[.]' <<<"$META" | sort -u | grep -vx "$1"
}

workspace_members() {
  jq -r '.workspace_members[] as $id | .packages[] | select(.id == $id) | .name' <<<"$META" | sort
}

FAILED=0
violation() {
  echo "NG: $1"
  FAILED=1
}

# ルール: crate 名のパターン → 依存してよい workspace crate のパターン(正規表現)
allowed_workspace_deps() {
  case "$1" in
    *-domain) echo '^platform-kernel$' ;;
    *-usecase) echo "^(${1%-usecase}-domain|platform-kernel)$" ;;
    *-infrastructure) echo "^(${1%-infrastructure}-(domain|usecase)|platform-kernel)$" ;;
    *-handler) echo "^(${1%-handler}-(domain|usecase)|platform-(kernel|auth|gen))$" ;;
    platform-kernel | platform-gen) echo '^$' ;;
    platform-auth | platform-telemetry) echo '^platform-kernel$' ;;
    *) echo '.*' ;; # app 層(bootstrap・server・lambda・migrate)は組み立て役なので何に依存してもよい
  esac
}

MEMBERS=$(workspace_members)

for crate in $MEMBERS; do
  pattern=$(allowed_workspace_deps "$crate")
  for dep in $(direct "$crate"); do
    if grep -qx "$dep" <<<"$MEMBERS" && ! [[ "$dep" =~ $pattern ]]; then
      violation "$crate → $dep は禁止(許可: $pattern)"
    fi
  done
done

# domain の純度: I/O 系 crate に推移的にも依存しない。リポジトリの trait は usecase にあるので async-trait も不要
IO_CRATES='^(tokio|sqlx|sqlx-core|tonic|hyper|reqwest|axum|mio|aws-config|aws-smithy-runtime|lambda_runtime|serde_json|async-trait|futures-core)$'
for crate in $(grep -- '-domain$' <<<"$MEMBERS"); do
  for dep in $(transitive "$crate"); do
    if [[ "$dep" =~ $IO_CRATES ]]; then
      violation "$crate が I/O 系 crate $dep に(推移的に)依存している"
    fi
  done
done

if [[ $FAILED -eq 0 ]]; then
  echo "依存ルール: OK ($(wc -w <<<"$MEMBERS") crates)"
fi
exit $FAILED
