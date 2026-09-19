variable "name" {
  description = "リソース名の接頭辞"
  type        = string
}

variable "env" {
  description = "環境名(dev/stg/prd)。シークレット名に使う"
  type        = string
}

variable "vpc_id" {
  description = "VPC ID"
  type        = string
}

variable "subnet_ids" {
  description = "DB を置くプライベートサブネット"
  type        = list(string)
}

variable "allowed_security_group_ids" {
  description = "3306 への接続を許可するセキュリティグループ(ECS タスク・Lambda)"
  type        = list(string)
}

variable "instance_class" {
  description = "インスタンスクラス"
  type        = string
  default     = "db.t4g.micro"
}

variable "multi_az" {
  description = "マルチ AZ にするか"
  type        = bool
  default     = false
}

variable "deletion_protection" {
  description = "削除保護。prd では true"
  type        = bool
  default     = false
}

variable "performance_insights_enabled" {
  description = "Performance Insights。db.t4g.micro・small では使えない(作成時にエラーになる)"
  type        = bool
  default     = false
}
