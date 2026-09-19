# modules/stack の出力をそのまま出す(deploy ワークフローが terraform output で読む)
output "auth_issuer" {
  value = module.stack.auth_issuer
}

output "cognito_client_id" {
  value = module.stack.cognito_client_id
}

output "cognito_user_pool_id" {
  value = module.stack.cognito_user_pool_id
}

output "queue_url" {
  value = module.stack.queue_url
}

output "database_url_secret_name" {
  value = module.stack.database_url_secret_name
}

output "payout_api_key_secret_name" {
  value = module.stack.payout_api_key_secret_name
}

output "alarm_topic_arn" {
  value = module.stack.alarm_topic_arn
}

output "ecr_repository_url" {
  value = module.stack.ecr_repository_url
}

output "ecs_cluster_name" {
  value = module.stack.ecs_cluster_name
}

output "ecs_service_name" {
  value = module.stack.ecs_service_name
}

output "migrate_task_definition_arn" {
  value = module.stack.migrate_task_definition_arn
}

output "migrate_network_configuration" {
  value = module.stack.migrate_network_configuration
}

output "web_bucket_name" {
  value = module.stack.web_bucket_name
}

output "web_distribution_id" {
  value = module.stack.web_distribution_id
}

output "alb_dns_name" {
  value = module.stack.alb_dns_name
}

output "lambda_artifact_bucket" {
  value = var.lambda_artifact_bucket
}
