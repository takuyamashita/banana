variable "name" {
  description = "リソース名の接頭辞(例: platform-dev)"
  type        = string
}

variable "cidr_block" {
  description = "VPC の CIDR"
  type        = string
  default     = "10.0.0.0/16"
}

variable "nat_gateway_per_az" {
  description = "NAT を AZ ごとに置くか。false なら1つ(安いが、その AZ の障害で外向き通信が止まる)"
  type        = bool
  default     = false
}
