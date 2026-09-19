variable "name" {
  description = "リソース名の接頭辞"
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

variable "public_subnet_ids" {
  description = "ALB を置くサブネット"
  type        = list(string)
}

variable "private_subnet_ids" {
  description = "タスクを置くサブネット"
  type        = list(string)
}

variable "image" {
  description = "server イメージ(ECR の URI:タグ)。migrate も同じイメージを使う"
  type        = string
}

variable "certificate_arn" {
  description = "ALB の HTTPS リスナーに付ける ACM 証明書"
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

variable "queue_arn" {
  description = "outbox relay が送る SQS キュー"
  type        = string
}

variable "user_pool_arn" {
  description = "UserDirectory が操作する Cognito ユーザープール"
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

variable "container_insights" {
  description = "Container Insights(タスクごとの詳しい指標。有料)を有効にするか"
  type        = bool
  default     = false
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

variable "deletion_protection" {
  description = "ALB の削除保護"
  type        = bool
  default     = false
}

variable "app_environment" {
  description = "アプリに渡す設定(APP__SECTION__KEY)。インフラが決める値(issuer・キュー URL など)を渡す"
  type        = map(string)
  default     = {}
}
