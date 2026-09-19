output "database_url_secret_arn" {
  value = aws_secretsmanager_secret.database_url.arn
}

output "database_url_secret_name" {
  value = aws_secretsmanager_secret.database_url.name
}

output "security_group_id" {
  value = aws_security_group.this.id
}

# 管理者のシークレット(RDS が管理する。JSON: username, password)。migrate だけに読ませる
output "admin_secret_arn" {
  value = aws_db_instance.this.master_user_secret[0].secret_arn
}

output "app_user_secret_arn" {
  value = aws_secretsmanager_secret.app_user.arn
}
