#!/usr/bin/env bash
# この PR の台本(e2e/videos/ で main から足した・変えたもの)で動作確認の動画を撮り、今のブランチの PR に貼る
# (mise run pr:video)。引数は Playwright に渡る(例: -- -g 管理者)。
# 初めて貼るときは PR の本文の末尾(テンプレートの「動作確認」の節)に、撮り直したときはコメントとして足す。
# 比べる先は PR_VIDEO_BASE で変えられる(既定は origin/main)
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

base="${PR_VIDEO_BASE:-origin/main}"
# 貼った印。本文にあれば、撮り直しとしてコメントに足す
marker="<!-- e2e:video -->"

# 撮る前に、今のブランチの PR があることを確かめる(撮った後に無いと分かると無駄になる)
if ! body="$(gh pr view --json body --jq .body)"; then
  echo "今のブランチの PR が見つからない。先に gh pr create で PR を作る" >&2
  exit 1
fi

mise run e2e:video -- --only-changed="$base" "$@"

attach=()
while IFS= read -r video; do
  # 代わりの文字(alt)は「台本 / テスト名」
  attach+=(--attach "$video#$(basename "$(dirname "$video")") / $(basename "$video" .mp4)")
done < <(find e2e/videos-out -name '*.mp4' | sort)
if [ ${#attach[@]} -eq 0 ]; then
  echo "撮った動画がない" >&2
  exit 1
fi

if grep -qF -- "$marker" <<<"$body"; then
  gh pr comment --body "動作確認の動画(撮り直し。mise run pr:video)" "${attach[@]}"
else
  # 貼った動画は本文の末尾に足される。その前に印と見出しを置く
  new_body="$(mktemp)"
  trap 'rm -f "$new_body"' EXIT
  printf '%s\n\n%s\n動作確認の動画(mise run pr:video):\n' "$body" "$marker" >"$new_body"
  gh pr edit --body-file "$new_body" "${attach[@]}"
fi
gh pr view --json url --jq .url
