# ---- 環境の名前 ----

variable "name" {
  description = "リソース名の接頭辞(例: platform-prod)"
  type        = string
}

variable "env" {
  description = "APP_ENV(config/{env}.toml)。dev・stg・prd"
  type        = string
}

variable "cidr_block" {
  description = "VPC の CIDR"
  type        = string
  default     = "10.0.0.0/16"
}

# ---- 環境ごとに外で決まる値(terraform.tfvars・deploy ワークフロー)----

variable "image_tag" {
  description = "server イメージのタグ(git sha)"
  type        = string
}

variable "lambda_artifact_bucket" {
  description = "Lambda の zip を置く S3 バケット"
  type        = string
}

variable "lambda_artifact_key" {
  description = "payout-dispatcher の zip の S3 キー"
  type        = string
}

variable "api_domain" {
  description = "API(ALB)のドメイン"
  type        = string
}

variable "web_domain" {
  description = "フロント(CloudFront)のドメイン"
  type        = string
}

variable "alb_certificate_arn" {
  description = "ALB 用 ACM 証明書(ap-northeast-1)"
  type        = string
}

variable "cloudfront_certificate_arn" {
  description = "CloudFront 用 ACM 証明書(us-east-1)"
  type        = string
}

variable "adot_layer_arn" {
  description = "ADOT collector の Lambda レイヤー ARN"
  type        = string
  default     = null
}

variable "alarm_email" {
  description = "アラームを受け取るメールアドレス。null なら購読を作らない(SNS トピックに別の通知先をつなぐ)"
  type        = string
  default     = null
}

# ---- 規模と保護の強さ(環境ごとの差はここだけ)----

variable "deletion_protection" {
  description = "DB・ALB・ユーザープールの削除保護"
  type        = bool
}

variable "db_instance_class" {
  description = "RDS のインスタンスクラス"
  type        = string
}

variable "db_multi_az" {
  description = "RDS をマルチ AZ にするか"
  type        = bool
}

variable "db_performance_insights_enabled" {
  description = "Performance Insights(db.t4g.micro・small では使えない)"
  type        = bool
  default     = false
}

variable "desired_count" {
  description = "server のタスク数の下限"
  type        = number
}

variable "max_count" {
  description = "server のタスク数の上限(オートスケール)"
  type        = number
}

variable "nat_gateway_per_az" {
  description = "NAT を AZ ごとに置くか"
  type        = bool
}

variable "container_insights" {
  description = "Container Insights を有効にするか(有料)"
  type        = bool
}

variable "mfa_configuration" {
  description = "Cognito の多要素認証(OFF・OPTIONAL・ON)"
  type        = string
}
