variable "name" {
  description = "リソース名の接頭辞(例: platform-dev)"
  type        = string
}

variable "cidr_block" {
  description = "VPC の CIDR"
  type        = string
  default     = "10.0.0.0/16"
}
