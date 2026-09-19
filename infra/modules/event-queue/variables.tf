variable "name" {
  description = "キュー名(.fifo は付けない)"
  type        = string
}

variable "visibility_timeout_seconds" {
  description = "可視性タイムアウト。受け手が Lambda ならそのタイムアウトの6倍以上。処理に失敗したメッセージは、この時間の後に再配信される"
  type        = number
  default     = 180
}

variable "alarm_actions" {
  description = "DLQ のアラームの通知先(SNS トピックの ARN など)。空ならアラームの状態だけが変わる"
  type        = list(string)
  default     = []
}

variable "subscriptions" {
  description = "購読する出来事のトピックと、受け取る出来事の種類(event_type)"
  type = list(object({
    topic_arn   = string
    event_types = list(string)
  }))
  default = []
}
