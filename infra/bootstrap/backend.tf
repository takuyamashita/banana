# 最初の1回は state の置き場がまだないので、ローカルの state で apply する。
# できたら下のコメントを外し、terraform init -migrate-state で作ったバケットに移す
# terraform {
#   backend "s3" {
#     bucket       = "platform-terraform-state-bootstrap"
#     key          = "bootstrap/terraform.tfstate"
#     region       = "ap-northeast-1"
#     encrypt      = true
#     use_lockfile = true
#   }
# }
