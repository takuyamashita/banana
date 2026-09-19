variable "name" {
  description = "リソース名の接頭辞"
  type        = string
}

variable "vpc_id" {
  description = "VPC ID"
  type        = string
}

variable "public_subnet_ids" {
  description = "ALB を置くサブネット"
  type        = list(string)
}

variable "certificate_arn" {
  description = "ALB の HTTPS リスナーに付ける ACM 証明書"
  type        = string
}

variable "container_insights" {
  description = "Container Insights(タスクごとの詳しい指標。有料)を有効にするか"
  type        = bool
  default     = false
}

variable "deletion_protection" {
  description = "ALB の削除保護"
  type        = bool
  default     = false
}

variable "alarm_actions" {
  description = "アラームの通知先(SNS トピックの ARN など)"
  type        = list(string)
  default     = []
}
