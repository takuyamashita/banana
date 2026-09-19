# 0005 環境の構成を1つのモジュールにまとめ、承認した plan だけを apply する

- 日付: 2026-09-19
- 状態: 採用

## 決定

- dev・stg・prod の構成は `infra/modules/stack` に1つにし、`infra/envs/<環境>` は値を渡すだけにする。環境の差は「規模と保護の強さ」の入力(削除保護・DB の大きさ・マルチ AZ・タスク数・NAT・Container Insights・MFA)に絞る。
- デプロイは、読み取りロールで作った plan を承認者が見てから、その plan を apply する(`deploy-environment.yml`)。ロールと state の置き場は `infra/bootstrap` でコードにする。
- DB は管理者(RDS が管理し、migrate だけが使う)とアプリ用(読み書きだけ。migrate が作る)のユーザーに分ける。

## 理由

3つの環境の main.tf をコピーで持つと、少しずつずれて差が読めなくなる。承認者が見た plan と違う変更が apply されると承認の意味がない。アプリが DDL を実行できると、不具合や乗っ取りでテーブルを消されうる。Terraform からは VPC 内の DB に届かないので、アプリ用ユーザーは VPC 内で動く migrate が作る。

## 影響

構成を動かしたので、既存の state は `moved` で付け替える(`envs/*/moved.tf`)。AWS への apply はまだ行っていない。
