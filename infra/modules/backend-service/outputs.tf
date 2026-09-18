output "ecr_repository_url" {
  value = aws_ecr_repository.this.repository_url
}

output "alb_dns_name" {
  value = aws_lb.this.dns_name
}

output "task_security_group_id" {
  value = aws_security_group.task.id
}

output "cluster_name" {
  value = aws_ecs_cluster.this.name
}

output "service_name" {
  value = aws_ecs_service.this.name
}

output "migrate_task_definition_arn" {
  value = aws_ecs_task_definition.migrate.arn
}
