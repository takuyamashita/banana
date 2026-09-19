#!/usr/bin/env bash
# 動作確認の動画を撮り、mp4 にする(mise run e2e:video。引数は Playwright に渡る。例: -g 管理者)。
# ffmpeg は mise のタスクが、このタスクを動かしたときだけ入れる
set -euo pipefail
cd "$(dirname "$0")/.."

rm -rf videos-out
pnpm exec playwright test --config playwright.video.config.ts "$@"

# 録画は webm(Playwright の形式)なので、どこでも再生できる H.264 の mp4 にする
for webm in videos-out/*.webm; do
  ffmpeg -hide_banner -loglevel error -y -i "$webm" \
    -c:v libx264 -pix_fmt yuv420p -crf 20 -movflags +faststart "${webm%.webm}.mp4"
  rm "$webm"
done
rm -rf videos-out/raw
echo "動画: $(pwd)/videos-out"
ls -1 videos-out/*.mp4
