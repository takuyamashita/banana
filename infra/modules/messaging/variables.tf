variable "name" {
  description = "キュー名(.fifo は付けない)"
  type        = string
}

variable "visibility_timeout_seconds" {
  description = "可視性タイムアウト。consumer Lambda のタイムアウトの6倍以上"
  type        = number
  default     = 180
}

variable "alarm_actions" {
  description = "DLQ のアラームの通知先(SNS トピックの ARN など)。空ならアラームの状態だけが変わる"
  type        = list(string)
  default     = []
}
