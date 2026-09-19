# GitHub Environment「<環境>」の変数 AWS_DEPLOY_ROLE_ARN に入れる
output "deploy_role_arns" {
  value = { for env, role in aws_iam_role.deploy : env => role.arn }
}

# GitHub Environment「<環境>-plan」の変数 AWS_PLAN_ROLE_ARN に入れる
output "plan_role_arns" {
  value = { for env, role in aws_iam_role.plan : env => role.arn }
}

output "artifact_bucket" {
  value = aws_s3_bucket.this[var.artifact_bucket].id
}
