variable "bucket_name" {
  description = "静的ファイルを置く S3 バケット名"
  type        = string
}

variable "aliases" {
  description = "CloudFront の代替ドメイン名"
  type        = list(string)
}

variable "certificate_arn" {
  description = "CloudFront 用の ACM 証明書(us-east-1)"
  type        = string
}

variable "runtime_config" {
  description = "フロントが実行時に読む config.json の中身"
  type = object({
    apiBaseUrl = string
    oidc = object({
      authority = string
      clientId  = string
      # ディスカバリに載っていない宛先を補う(Cognito の /logout・/oauth2/revoke)
      endSessionEndpoint      = optional(string)
      revocationEndpoint      = optional(string)
      postLogoutRedirectParam = optional(string)
    })
  })
}

variable "extra_connect_origins" {
  description = "config.json に載らないが画面が話す相手のオリジン(ディスカバリが別のホストのトークン発行先を返す IdP など)"
  type        = list(string)
  default     = []
}
