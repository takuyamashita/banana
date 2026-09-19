#!/usr/bin/env bash
# この PR の台本(e2e/videos/ で main から足した・変えたもの)で動作確認の動画を撮り、今のブランチの PR に貼る
# (mise run pr:video)。引数は Playwright に渡る(例: -- -g 管理者)。
# 初めて貼るときは PR の本文の「動作確認」の節に、撮り直したときはコメントとして足す。
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

videos=()
while IFS= read -r video; do videos+=("$video"); done < <(find e2e/videos-out -name '*.mp4' | sort)
if [ ${#videos[@]} -eq 0 ]; then
  echo "撮った動画がない" >&2
  exit 1
fi

# 貼る動画の並び。本文の ![](<パス>) を gh がアップロード後の URL に置き換え、動画のプレイヤーとして出る
# (1段落に1つだけ置く。動画には alt を付けられない。台本とテストの名前はファイル名に入っている)
block="$(mktemp)"
new_body="$(mktemp)"
trap 'rm -f "$block" "$new_body"' EXIT
attach=()
for video in "${videos[@]}"; do
  printf '\n![](%s)\n' "$video" >>"$block"
  attach+=(--attach "$video")
done

if grep -qF -- "$marker" <<<"$body"; then
  { printf '動作確認の動画(撮り直し。mise run pr:video)\n'; cat "$block"; } >"$new_body"
  gh pr comment --body-file "$new_body" "${attach[@]}"
else
  # 本文の「## 動作確認」の節(テンプレートの説明の直後)に置く。その節がなければ末尾に置く
  { printf '%s\n**動作確認の動画**(mise run pr:video)\n' "$marker"; cat "$block"; } >"$block.full"
  mv "$block.full" "$block"
  awk -v blockfile="$block" '
    function emit() { print ""; while ((getline line < blockfile) > 0) print line; close(blockfile); done = 1 }
    { print }
    done { next }
    state == 0 && /^## 動作確認/ { state = 1; next }
    state == 1 && /^## / { state = 2; next }
    state == 1 && /-->/ { emit() }
    END { if (!done) emit() }
  ' <<<"$body" >"$new_body"
  gh pr edit --body-file "$new_body" "${attach[@]}"
fi
gh pr view --json url --jq .url
