variable "name" {
  description = "キュー名(.fifo は付けない)"
  type        = string
}

variable "visibility_timeout_seconds" {
  description = "可視性タイムアウト。consumer Lambda のタイムアウトの6倍以上"
  type        = number
  default     = 180
}
