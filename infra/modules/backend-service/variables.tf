variable "name" {
  description = "リソース名の接頭辞"
  type        = string
}

variable "env_prefix" {
  description = "アプリの設定を上書きする環境変数の接頭辞(PAYROLL なら PAYROLL__SECTION__KEY)"
  type        = string
}

variable "env" {
  description = "APP_ENV に渡す環境名(dev/stg/prd)"
  type        = string
}

variable "vpc_id" {
  description = "VPC ID"
  type        = string
}

variable "database_security_group_id" {
  description = "DB のセキュリティグループ。MySQL への外向き通信をここだけに限る"
  type        = string
}

variable "private_subnet_ids" {
  description = "タスクを置くサブネット"
  type        = list(string)
}

variable "cluster_id" {
  description = "タスクを動かす ECS クラスター(load-balancer)"
  type        = string
}

variable "cluster_name" {
  description = "タスクを動かす ECS クラスターの名前(オートスケールの対象の指定に使う)"
  type        = string
}

variable "listener_arn" {
  description = "共有の ALB の HTTPS リスナー。このサービスへの振り分けのルールを足す"
  type        = string
}

variable "listener_rule_priority" {
  description = "振り分けのルールの優先順位(サービスごとに別の値)"
  type        = number
}

variable "path_patterns" {
  description = "このサービスに振り分ける RPC のパス(例: /acme.timesheet.v1.*)"
  type        = list(string)
}

variable "alb_security_group_id" {
  description = "共有の ALB のセキュリティグループ。ここからタスクへの通信を許す"
  type        = string
}

variable "alb_arn_suffix" {
  description = "共有の ALB(アラームの指標の指定に使う)"
  type        = string
}

variable "task_policy_json" {
  description = "server のタスクに足す権限(出来事の送り先・自分のキュー・認証基盤など)。IAM ポリシーの JSON"
  type        = string
  default     = null
}

variable "image" {
  description = "server イメージ(ECR の URI:タグ)。migrate も同じイメージを使う"
  type        = string
}

variable "database_url_secret_arn" {
  description = "DB 接続文字列のシークレット(アプリ用のユーザー)"
  type        = string
}

variable "database_admin_secret_arn" {
  description = "DB の管理者のシークレット。migrate だけが読む"
  type        = string
}

variable "database_app_user_secret_arn" {
  description = "migrate が作るアプリ用の DB ユーザーのシークレット"
  type        = string
}

variable "cpu" {
  description = "タスクの CPU ユニット"
  type        = number
  default     = 512
}

variable "memory" {
  description = "タスクのメモリ(MiB)"
  type        = number
  default     = 1024
}

variable "desired_count" {
  description = "タスク数の下限(オートスケールはここから増やす)"
  type        = number
  default     = 2
}

variable "max_count" {
  description = "オートスケールで増やすタスク数の上限。タスク数 × database.max_connections が RDS の上限に収まるようにする"
  type        = number
  default     = 4
}

variable "ecr_keep_images" {
  description = "ECR に残すイメージの数(ロールバックに使う)"
  type        = number
  default     = 30
}

variable "alarm_actions" {
  description = "アラームの通知先(SNS トピックの ARN など)"
  type        = list(string)
  default     = []
}

variable "adot_collector_version" {
  description = "ADOT collector のイメージタグ"
  type        = string
  default     = "v0.50.0"
}

variable "app_environment" {
  description = "アプリに渡す設定(APP__SECTION__KEY)。インフラが決める値(issuer・キュー URL など)を渡す"
  type        = map(string)
  default     = {}
}
