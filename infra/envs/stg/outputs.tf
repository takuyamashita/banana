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

output "payroll_events_topic_arn" {
  value = module.stack.payroll_events_topic_arn
}

output "timesheet_events_topic_arn" {
  value = module.stack.timesheet_events_topic_arn
}

output "database_url_secret_names" {
  value = module.stack.database_url_secret_names
}

output "payout_api_key_secret_name" {
  value = module.stack.payout_api_key_secret_name
}

output "alarm_topic_arn" {
  value = module.stack.alarm_topic_arn
}

output "ecs_cluster_name" {
  value = module.stack.ecs_cluster_name
}

output "services" {
  value = module.stack.services
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
