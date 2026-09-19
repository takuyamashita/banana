variable "image_tag" {
  description = "server イメージのタグ(git sha)。deploy ワークフローが渡す"
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
  description = "アラームを受け取るメールアドレス。null なら購読を作らない"
  type        = string
  default     = null
}
