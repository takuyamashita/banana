# 確かめ用。アプリには main.tf の locals から環境変数で渡すので、転記は要らない
output "auth_issuer" {
  value = module.auth.issuer
}

output "cognito_client_id" {
  value = module.auth.web_client_id
}

output "cognito_user_pool_id" {
  value = module.auth.user_pool_id
}

output "queue_url" {
  value = module.messaging.queue_url
}

output "database_url_secret_name" {
  value = module.database.database_url_secret_name
}

output "payout_api_key_secret_name" {
  value = aws_secretsmanager_secret.payout_api_key.name
}

output "alarm_topic_arn" {
  value = aws_sns_topic.alarms.arn
}

# deploy ワークフローが使う値
output "ecr_repository_url" {
  value = module.backend.ecr_repository_url
}

output "ecs_cluster_name" {
  value = module.backend.cluster_name
}

output "ecs_service_name" {
  value = module.backend.service_name
}

output "migrate_task_definition_arn" {
  value = module.backend.migrate_task_definition_arn
}

output "migrate_network_configuration" {
  value = module.backend.migrate_network_configuration
}

output "web_bucket_name" {
  value = module.frontend.bucket_name
}

output "web_distribution_id" {
  value = module.frontend.distribution_id
}

output "alb_dns_name" {
  value = module.backend.alb_dns_name
}
