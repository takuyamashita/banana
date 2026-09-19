variable "github_repository" {
  description = "デプロイする GitHub のリポジトリ(owner/name)"
  type        = string
  default     = "takuyamashita/banana"
}

variable "environments" {
  description = "環境の名前(GitHub Environments の名前と infra/envs/ のディレクトリ名)"
  type        = list(string)
  default     = ["dev", "stg", "prod"]
}

variable "artifact_bucket" {
  description = "Lambda の zip を置くバケット(各環境の terraform.tfvars の lambda_artifact_bucket)"
  type        = string
}
