# state は環境ごとに S3 で分離する。use_lockfile で S3 ネイティブのロックを使う(DynamoDB 不要)
terraform {
  backend "s3" {
    bucket       = "platform-terraform-state-dev"
    key          = "platform/terraform.tfstate"
    region       = "ap-northeast-1"
    encrypt      = true
    use_lockfile = true
  }
}
