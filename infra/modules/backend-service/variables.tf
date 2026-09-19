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

variable "vpc_cidr" {
  description = "VPC の CIDR(MySQL への外向き通信を VPC 内に限る)"
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
  description = "DB 接続文字列のシークレット"
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
  description = "タスク数"
  type        = number
  default     = 2
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
