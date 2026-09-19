# バケット名は全 AWS で一意にする。000000000000 を実際のアカウント ID に置き換える
# (各環境の terraform.tfvars の lambda_artifact_bucket も同じ名前にする)
artifact_bucket = "platform-artifacts-000000000000"
