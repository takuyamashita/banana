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

output "payroll_events_topic_arn" {
  value = module.payroll_events.arn
}

output "timesheet_events_topic_arn" {
  value = module.timesheet_events.arn
}

output "database_url_secret_names" {
  value = module.database.database_url_secret_names
}

output "payout_api_key_secret_name" {
  value = aws_secretsmanager_secret.payout_api_key.name
}

output "alarm_topic_arn" {
  value = aws_sns_topic.alarms.arn
}

# deploy ワークフローが使う値
output "ecs_cluster_name" {
  value = module.load_balancer.cluster_name
}

# サービスごとのイメージの置き場・ECS サービス・migrate。deploy ワークフローはサービスごとに繰り返す
# (JSON。キーは backend/Dockerfile の SERVICE と同じ)
output "services" {
  value = jsonencode({
    for name, service in { payroll  = module.payroll, timesheet = module.timesheet } : name => {
      ecr_repository_url            = service.ecr_repository_url
      ecs_service_name              = service.service_name
      migrate_task_definition_arn   = service.migrate_task_definition_arn
      migrate_network_configuration = service.migrate_network_configuration
    }
  })
}

output "web_bucket_name" {
  value = module.frontend.bucket_name
}

output "web_distribution_id" {
  value = module.frontend.distribution_id
}

output "alb_dns_name" {
  value = module.load_balancer.alb_dns_name
}
