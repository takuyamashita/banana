output "database_url_secret_arn" {
  value = aws_secretsmanager_secret.database_url.arn
}

output "database_url_secret_name" {
  value = aws_secretsmanager_secret.database_url.name
}

output "security_group_id" {
  value = aws_security_group.this.id
}
