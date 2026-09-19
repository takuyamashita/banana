#!/bin/bash
# Claude Code on the web のセッションが始まる前に、README のセットアップを済ませておく。
# 手元の CLI では何もしない(手元は README の手順が正)。
#
# このフックが終わった時点のコンテナの状態がキャッシュされるので、重い導入
# (cargo install・docker pull・cargo fetch)を払うのは環境を作り直したときだけで済む。
# 何度走らせても安全なように、入っていないものだけを入れる。
#
# 版はできるだけ mise.toml と compose.yaml から読み、この中に写さない(ずれないように)。
set -euo pipefail

# 手元の CLI セッションでは走らせない
[ "${CLAUDE_CODE_REMOTE:-}" = "true" ] || exit 0

cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel)}"

log() { printf '[session-start] %s\n' "$*"; }
# 失敗してもセッションは続けたい処理(ネットワーク次第のもの)に使う
optional() { "$@" || log "スキップ(失敗): $*"; }

# mise.toml に書かれたツールの版。見つからなければ止める(黙って古い版を入れないため)
pinned() {
  local version
  version=$(grep -m1 "^\"\?$1\"\? *=" mise.toml | grep -oP '(?<=")[0-9][^"]*(?=")' | head -1)
  [ -n "$version" ] && printf '%s' "$version" || { log "mise.toml に $1 の版がない"; return 1; }
}

export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
export MISE_DISABLE_VERSION_CHECK=1
export DEBIAN_FRONTEND=noninteractive

# ---- 1. OS のパッケージ -------------------------------------------------
# build-essential・pkg-config は README の前提。ffmpeg は動作確認の動画の mp4 変換
# (mise.toml は conda の ffmpeg を使うが anaconda.org に届かないので apt のものを使う)。
# 日本語フォントがないと E2E のブラウザで文字化けする
missing=()
for pkg in build-essential pkg-config ffmpeg fonts-noto-cjk; do
  dpkg -s "$pkg" >/dev/null 2>&1 || missing+=("$pkg")
done
if [ ${#missing[@]} -gt 0 ]; then
  log "apt: ${missing[*]}"
  optional apt-get update -qq
  optional apt-get install -y -qq "${missing[@]}"
fi

# ---- 2. mise 本体 -------------------------------------------------------
# mise.jdx.dev のインストーラではなく GitHub のリリースから直接取る
# (api.github.com と違い releases/download/ は塞がれないため、どの設定でも通る)。
# 上げるときはこの版を書き換える。ツールの版は mise.toml が持つ
MISE_VERSION=2026.9.5
if ! command -v mise >/dev/null; then
  log "mise $MISE_VERSION を入れる"
  tmp=$(mktemp -d)
  curl -fsSL -o "$tmp/mise.tar.gz" \
    "https://github.com/jdx/mise/releases/download/v$MISE_VERSION/mise-v$MISE_VERSION-linux-x64.tar.gz"
  tar xzf "$tmp/mise.tar.gz" -C "$tmp"
  mkdir -p "$HOME/.local/bin"
  install -m755 "$tmp/mise/bin/mise" "$HOME/.local/bin/mise"
  rm -rf "$tmp"
fi

# ---- 3. mise が扱えないツールを外す -------------------------------------
# api.github.com は「このセッションのリポジトリ以外は不可」で 403 になる。
# GitHub API でしか版を解決できないバックエンド(github:・cargo-binstall)だけ無効にし、
# 同じ版を下で手で入れる。conda は anaconda.org に届かないので apt の ffmpeg を使う。
# リポジトリの mise.toml は変えない
mkdir -p "$HOME/.config/mise"
if [ ! -f "$HOME/.config/mise/config.toml" ]; then
  cat > "$HOME/.config/mise/config.toml" <<'EOF'
# Claude Code on the web 用。.claude/hooks/session-start.sh が作る。
# api.github.com が塞がれていて mise が版を解決できないツールだけ無効にし、
# 同じ版を手で入れて PATH に置く
[settings]
disable_update_warning = true
disable_tools = [
  "conda:ffmpeg",
  "github:reteps/dockerfmt",
  "cargo-binstall",
  "cargo:sqlx-cli",
  "cargo:cargo-nextest",
  "cargo:cargo-llvm-cov",
  "cargo:cargo-deny",
  "cargo:cargo-lambda",
]
EOF
fi

# ---- 4. .env と mise のツール一式 ---------------------------------------
[ -f .env ] || cp .env.example .env
mise trust >/dev/null 2>&1 || true
log "mise install"
mise install
# shims(どのシェルからでも引ける入口)を作り直す。下で PATH に入れる
mise reshim

# 以降、mise のツール(node・pnpm・lefthook…)を使えるようにする
eval "$(mise env -s bash)"

# ---- 5. mise で入らないツール -------------------------------------------
# dockerfmt: github: バックエンドが使えないので、同じ版を go で入れる(go はこの環境に既にある)
if ! command -v dockerfmt >/dev/null && command -v go >/dev/null; then
  log "dockerfmt $(pinned 'github:reteps/dockerfmt') を入れる"
  optional env GOBIN="$HOME/.local/bin" GOFLAGS=-mod=mod \
    go install "github.com/reteps/dockerfmt@v$(pinned 'github:reteps/dockerfmt')"
fi

# cargo 製のツール: cargo-binstall が使えないので crates.io からビルドする。
# 一番時間がかかる(初回で20〜30分)が、その分がキャッシュされる
cargo_install() { # cargo_install <コマンド名> <mise.toml のキー> [追加の引数…]
  local bin=$1 key=$2 version
  shift 2
  command -v "$bin" >/dev/null && return 0
  version=$(pinned "$key") || return 1
  log "cargo install $bin@$version"
  optional cargo install "${key#cargo:}@$version" --locked "$@"
}
# sqlx-cli は既定の native-tls を避け、MySQL + rustls に絞る(mise.toml と同じ)
cargo_install sqlx cargo:sqlx-cli --no-default-features --features mysql,rustls,completions
cargo_install cargo-nextest cargo:cargo-nextest
cargo_install cargo-deny cargo:cargo-deny
cargo_install cargo-llvm-cov cargo:cargo-llvm-cov
cargo_install cargo-lambda cargo:cargo-lambda

# ---- 6. Docker ----------------------------------------------------------
# デーモンはセッションごとに立ち上げ直す(プロセスなのでキャッシュに残らない)
if ! docker info >/dev/null 2>&1; then
  # Docker Hub の blob の CDN(production.cloudfront.docker.com)が塞がれているので、
  # mirror.gcr.io 経由で取る
  mkdir -p /etc/docker
  [ -f /etc/docker/daemon.json ] || echo '{"registry-mirrors":["https://mirror.gcr.io"]}' > /etc/docker/daemon.json
  # 前に落ちていると pid のファイルだけが残り、dockerd が起動を断る
  if [ -f /var/run/docker.pid ] && ! kill -0 "$(cat /var/run/docker.pid)" 2>/dev/null; then
    rm -f /var/run/docker.pid
  fi
  log "dockerd を起動する"
  nohup dockerd >/var/log/dockerd.log 2>&1 &
  for _ in $(seq 1 60); do docker info >/dev/null 2>&1 && break; sleep 1; done
  docker info >/dev/null 2>&1 || log "dockerd が起動しなかった(/var/log/dockerd.log を見る)"
fi

# compose のイメージを先に取っておく(testcontainers の MySQL も compose と同じ版を使う)
if docker info >/dev/null 2>&1; then
  while read -r image; do
    docker image inspect "$image" >/dev/null 2>&1 && continue
    log "docker pull $image"
    optional docker pull -q "$image"
  done < <(grep -oP '(?<=^    image: ).*' compose.yaml)
fi

# ---- 7. フロントエンドと Rust の依存 ------------------------------------
log "pnpm install"
pnpm install
lefthook install >/dev/null

# crates を落としておく(ビルド自体はしない。target/ はクローンし直しで消えるため)
log "cargo fetch"
optional cargo fetch --quiet

# Playwright: この環境に入っている Chromium は版が古いことがあるので、
# @playwright/test が要求するビルドを入れる(既にあれば何もしない)
log "playwright install chromium"
optional env PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=0 pnpm --filter e2e exec playwright install chromium

# ---- 8. セッションに環境変数を渡す --------------------------------------
# PATH は mise の shims(どのシェルでも効く)。mise.toml の [env](DATABASE_URL・
# SQLX_OFFLINE など)もそのまま渡す。PATH だけは shims を使うので除く
if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
  {
    echo 'export MISE_DISABLE_VERSION_CHECK=1'
    echo 'export PATH="$HOME/.local/share/mise/shims:$HOME/.local/bin:$HOME/.cargo/bin:$PATH"'
    mise env -s bash | grep -v '^export PATH='
  } >> "$CLAUDE_ENV_FILE"
fi

log "完了"
