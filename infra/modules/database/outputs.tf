output "security_group_id" {
  value = aws_security_group.this.id
}

# 管理者のシークレット(RDS が管理する。JSON: username, password)。各サービスの migrate だけに読ませる
output "admin_secret_arn" {
  value = aws_db_instance.this.master_user_secret[0].secret_arn
}

# サービス名 → 接続文字列のシークレット
output "database_url_secret_arns" {
  value = { for service, secret in aws_secretsmanager_secret.database_url : service => secret.arn }
}

output "database_url_secret_names" {
  value = { for service, secret in aws_secretsmanager_secret.database_url : service => secret.name }
}

# サービス名 → migrate が作るアプリ用のユーザーのシークレット
output "app_user_secret_arns" {
  value = { for service, secret in aws_secretsmanager_secret.app_user : service => secret.arn }
}
