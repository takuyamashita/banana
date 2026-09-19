variable "name" {
  description = "関数名"
  type        = string
}

variable "env" {
  description = "APP_ENV に渡す環境名"
  type        = string
}

variable "artifact_bucket" {
  description = "zip を置いた S3 バケット"
  type        = string
}

variable "artifact_key" {
  description = "zip の S3 キー(例: payout-dispatcher/<git sha>.zip)"
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

variable "subnet_ids" {
  description = "Lambda を置くプライベートサブネット"
  type        = list(string)
}

variable "memory_size" {
  description = "メモリ(MB)"
  type        = number
  default     = 256
}

variable "timeout" {
  description = "タイムアウト(秒)。SQS の可視性タイムアウトはこの6倍以上にする"
  type        = number
  default     = 30
}

variable "environment" {
  description = "追加の環境変数"
  type        = map(string)
  default     = {}
}

variable "sqs_queue_arn" {
  description = "トリガーにする SQS キュー。null ならトリガーなし"
  type        = string
  default     = null
}

variable "policy_json" {
  description = "追加で付与する IAM ポリシー(JSON)"
  type        = string
  default     = null
}

variable "adot_layer_arn" {
  description = "ADOT collector の Lambda レイヤー ARN(リージョン・アーキテクチャごとに異なる)"
  type        = string
  default     = null
}

variable "maximum_concurrency" {
  description = "SQS トリガーで同時に動かす数の上限(2 以上)。1つあたりの DB 接続数 × この値が RDS の上限に収まるようにする"
  type        = number
  default     = 2
}
