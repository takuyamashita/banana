#!/usr/bin/env bash
# 動作確認の動画を撮り、mp4 にする(mise run e2e:video)。引数は Playwright に渡る。
#   mise run e2e:video                                   台本(videos/)をすべて撮る
#   mise run e2e:video -- videos/<台本>.spec.ts          指定した台本だけ撮る
#   mise run e2e:video -- --only-changed=origin/main     この PR で足した・変えた台本だけ撮る
# ffmpeg は mise のタスクが、このタスクを動かしたときだけ入れる
set -euo pipefail
cd "$(dirname "$0")/.."

rm -rf videos-out
pnpm exec playwright test --config playwright.video.config.ts "$@"

# 録画は webm(Playwright の形式)なので、どこでも再生できる H.264 の mp4 にする
find videos-out -path videos-out/raw -prune -o -name '*.webm' -print | while read -r webm; do
  ffmpeg -nostdin -hide_banner -loglevel error -y -i "$webm" \
    -c:v libx264 -pix_fmt yuv420p -crf 20 -movflags +faststart "${webm%.webm}.mp4"
  rm "$webm"
done
rm -rf videos-out/raw
echo "動画: $(pwd)/videos-out"
find videos-out -name '*.mp4' | sort
