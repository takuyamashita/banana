# 0003 フロントの lint・整形に oxlint・oxfmt を使う

- 日付: 2026-09-19
- 状態: 採用

## 決定

ESLint・Prettier の代わりに oxlint(`--type-aware`)と oxfmt を使う。型検査は当面 `tsc --noEmit` を併用する。

## 理由

速い(このリポジトリ全体で数百ミリ秒)。設定が1ファイルで済み、features/ 間の import 禁止も `no-restricted-imports` の overrides で書ける。
