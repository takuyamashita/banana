# config/{env}.toml に転記する値
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

output "web_bucket_name" {
  value = module.frontend.bucket_name
}

output "web_distribution_id" {
  value = module.frontend.distribution_id
}

output "alb_dns_name" {
  value = module.backend.alb_dns_name
}
