variable "name" {
  description = "ユーザープール名"
  type        = string
}

variable "domain_prefix" {
  description = "Hosted UI のドメイン接頭辞(<prefix>.auth.<region>.amazoncognito.com)"
  type        = string
}

variable "callback_urls" {
  description = "ログイン後・ログアウト後に戻る URL(フロントの origin + /)"
  type        = list(string)
}

variable "deletion_protection" {
  description = "削除保護"
  type        = bool
  default     = false
}

variable "mfa_configuration" {
  description = "多要素認証。OFF・OPTIONAL(利用者が選ぶ)・ON(必須)"
  type        = string
  default     = "OPTIONAL"
  validation {
    condition     = contains(["OFF", "OPTIONAL", "ON"], var.mfa_configuration)
    error_message = "OFF・OPTIONAL・ON のどれか"
  }
}
