# 1つのサービス(給与・勤怠など)の gRPC(+ gRPC-Web)サーバーを ECS Fargate で動かす。
# 共有の ALB(load-balancer)のリスナーに、このサービスの RPC のパスを振り分けるルールを足す。
# 同じイメージに migrate も入れ、デプロイ前に単発タスクとして実行する
data "aws_region" "current" {}

resource "aws_ecr_repository" "this" {
  name                 = var.name
  image_tag_mutability = "IMMUTABLE"
  image_scanning_configuration {
    scan_on_push = true
  }
  encryption_configuration {
    encryption_type = "KMS"
  }
}

# 古いイメージを消す。ロールバックに使う最近の版は残す(タグは git sha で、上書きできない)
resource "aws_ecr_lifecycle_policy" "this" {
  repository = aws_ecr_repository.this.name
  policy = jsonencode({
    rules = [{
      rulePriority = 1
      description  = "keep the latest ${var.ecr_keep_images} images"
      selection = {
        tagStatus   = "any"
        countType   = "imageCountMoreThan"
        countNumber = var.ecr_keep_images
      }
      action = { type = "expire" }
    }]
  })
}

resource "aws_cloudwatch_log_group" "this" {
  name              = "/ecs/${var.name}"
  retention_in_days = 30
}

# ---- IAM ----

data "aws_iam_policy_document" "ecs_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["ecs-tasks.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "execution" {
  name               = "${var.name}-execution"
  assume_role_policy = data.aws_iam_policy_document.ecs_assume.json
}

resource "aws_iam_role_policy_attachment" "execution" {
  role       = aws_iam_role.execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

resource "aws_iam_role" "task" {
  name               = "${var.name}-task"
  assume_role_policy = data.aws_iam_policy_document.ecs_assume.json
}

data "aws_iam_policy_document" "task" {
  # サービスごとの権限(出来事の送り先・自分のキュー・認証基盤など)
  source_policy_documents = var.task_policy_json == null ? [] : [var.task_policy_json]
  # 起動時に DB 接続文字列を読む
  statement {
    actions   = ["secretsmanager:GetSecretValue"]
    resources = [var.database_url_secret_arn]
  }
  # ADOT collector がトレースを X-Ray に送る
  statement {
    actions   = ["xray:PutTraceSegments", "xray:PutTelemetryRecords", "xray:GetSamplingRules", "xray:GetSamplingTargets"]
    resources = ["*"]
  }
}

# migrate は server と別のロールにする。DB の管理者のシークレットを読めるのは migrate だけ
resource "aws_iam_role" "migrate" {
  name               = "${var.name}-migrate"
  assume_role_policy = data.aws_iam_policy_document.ecs_assume.json
}

data "aws_iam_policy_document" "migrate" {
  statement {
    actions   = ["secretsmanager:GetSecretValue"]
    resources = [var.database_url_secret_arn, var.database_admin_secret_arn, var.database_app_user_secret_arn]
  }
}

resource "aws_iam_role_policy" "migrate" {
  name   = "${var.name}-migrate"
  role   = aws_iam_role.migrate.id
  policy = data.aws_iam_policy_document.migrate.json
}

resource "aws_iam_role_policy" "task" {
  name   = "${var.name}-task"
  role   = aws_iam_role.task.id
  policy = data.aws_iam_policy_document.task.json
}

# ---- ネットワーク ----

# 共有の ALB から、このサービスのタスクへ
resource "aws_vpc_security_group_egress_rule" "alb_to_task" {
  security_group_id            = var.alb_security_group_id
  referenced_security_group_id = aws_security_group.task.id
  ip_protocol                  = "tcp"
  from_port                    = 50051
  to_port                      = 50051
  description                  = "To server"
}

resource "aws_security_group" "task" {
  name        = "${var.name}-task"
  description = "ECS tasks for ${var.name}"
  vpc_id      = var.vpc_id
}

resource "aws_vpc_security_group_ingress_rule" "task_from_alb" {
  security_group_id            = aws_security_group.task.id
  referenced_security_group_id = var.alb_security_group_id
  ip_protocol                  = "tcp"
  from_port                    = 50051
  to_port                      = 50051
  description                  = "From ALB"
}

# 外向きは HTTPS(AWS API・JWKS・外部 API)と DB だけ
resource "aws_vpc_security_group_egress_rule" "task_https" {
  security_group_id = aws_security_group.task.id
  cidr_ipv4         = "0.0.0.0/0"
  ip_protocol       = "tcp"
  from_port         = 443
  to_port           = 443
  description       = "AWS APIs, JWKS"
}

resource "aws_vpc_security_group_egress_rule" "task_mysql" {
  security_group_id            = aws_security_group.task.id
  referenced_security_group_id = var.database_security_group_id
  ip_protocol                  = "tcp"
  from_port                    = 3306
  to_port                      = 3306
  description                  = "MySQL"
}

# ブラウザからは gRPC-Web(HTTP/1.1 か HTTP/2 の通常の POST)で来るので、ターゲットは HTTP1 でよい。
# ネイティブ gRPC クライアントも受けるなら protocol_version = "GRPC" の別ターゲットグループが要る
resource "aws_lb_target_group" "this" {
  name                 = var.name
  port                 = 50051
  protocol             = "HTTP"
  protocol_version     = "HTTP1"
  target_type          = "ip"
  vpc_id               = var.vpc_id
  deregistration_delay = 30
  health_check {
    path                = "/health"
    matcher             = "200"
    interval            = 15
    healthy_threshold   = 2
    unhealthy_threshold = 3
  }
}

# このサービスの RPC(/<proto のパッケージ>.<サービス>/<メソッド>)を、このサービスに振り分ける
resource "aws_lb_listener_rule" "this" {
  listener_arn = var.listener_arn
  priority     = var.listener_rule_priority
  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.this.arn
  }
  condition {
    path_pattern {
      values = var.path_patterns
    }
  }
}

# ---- タスク定義 ----

locals {
  environment = concat(
    [
      { name = "APP_ENV", value = var.env },
      { name = "AWS_REGION", value = data.aws_region.current.region },
    ],
    [for name, value in var.app_environment : { name = name, value = value }],
  )
  log_configuration = {
    logDriver = "awslogs"
    options = {
      awslogs-group         = aws_cloudwatch_log_group.this.name
      awslogs-region        = data.aws_region.current.region
      awslogs-stream-prefix = "ecs"
    }
  }
}

resource "aws_ecs_task_definition" "server" {
  family                   = "${var.name}-server"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = var.cpu
  memory                   = var.memory
  execution_role_arn       = aws_iam_role.execution.arn
  task_role_arn            = aws_iam_role.task.arn
  runtime_platform {
    operating_system_family = "LINUX"
    cpu_architecture        = "ARM64"
  }
  container_definitions = jsonencode([
    {
      name         = "server"
      image        = var.image
      essential    = true
      portMappings = [{ containerPort = 50051, protocol = "tcp" }]
      environment  = local.environment
      # server の shutdown_grace_seconds(20秒)より長くし、SIGTERM 後に処理中のリクエストを終えられるようにする
      stopTimeout      = 30
      logConfiguration = local.log_configuration
      dependsOn        = [{ containerName = "otel-collector", condition = "START" }]
    },
    {
      # アプリは OTLP を localhost:4317 に出すだけ。送り先(X-Ray)はこの collector の設定に閉じる
      name             = "otel-collector"
      image            = "public.ecr.aws/aws-observability/aws-otel-collector:${var.adot_collector_version}"
      essential        = false
      command          = ["--config=/etc/ecs/ecs-default-config.yaml"]
      logConfiguration = local.log_configuration
    },
  ])
}

resource "aws_ecs_task_definition" "migrate" {
  family                   = "${var.name}-migrate"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = 256
  memory                   = 512
  execution_role_arn       = aws_iam_role.execution.arn
  task_role_arn            = aws_iam_role.migrate.arn
  runtime_platform {
    operating_system_family = "LINUX"
    cpu_architecture        = "ARM64"
  }
  container_definitions = jsonencode([
    {
      name       = "migrate"
      image      = var.image
      essential  = true
      entryPoint = ["/app/migrate"]
      environment = concat(local.environment, [
        { name = "${var.env_prefix}__TELEMETRY__OTLP_ENDPOINT", value = "" },
        { name = "${var.env_prefix}__SECRETS__DATABASE_ADMIN_SECRET_ID", value = var.database_admin_secret_arn },
        { name = "${var.env_prefix}__SECRETS__DATABASE_APP_USER_SECRET_ID", value = var.database_app_user_secret_arn },
      ])
      logConfiguration = local.log_configuration
    },
  ])
}

resource "aws_ecs_service" "this" {
  name                               = "${var.name}-server"
  cluster                            = var.cluster_id
  task_definition                    = aws_ecs_task_definition.server.arn
  desired_count                      = var.desired_count
  launch_type                        = "FARGATE"
  health_check_grace_period_seconds  = 30
  deployment_minimum_healthy_percent = 100
  deployment_maximum_percent         = 200
  deployment_circuit_breaker {
    enable   = true
    rollback = true
  }
  network_configuration {
    subnets         = var.private_subnet_ids
    security_groups = [aws_security_group.task.id]
  }
  load_balancer {
    target_group_arn = aws_lb_target_group.this.arn
    container_name   = "server"
    container_port   = 50051
  }
  depends_on = [aws_lb_listener_rule.this]
  # タスク数はオートスケールが決める(apply で最初の数に戻さない)
  lifecycle {
    ignore_changes = [desired_count]
  }
}

# ---- オートスケール ----

# CPU を目安に min_count〜max_count の間で増減する。DB の接続数(タスク数 × database.max_connections)が
# RDS の上限に収まるように max_count を決める
resource "aws_appautoscaling_target" "this" {
  service_namespace  = "ecs"
  resource_id        = "service/${var.cluster_name}/${aws_ecs_service.this.name}"
  scalable_dimension = "ecs:service:DesiredCount"
  min_capacity       = var.desired_count
  max_capacity       = var.max_count
}

resource "aws_appautoscaling_policy" "cpu" {
  name               = "${var.name}-cpu"
  service_namespace  = aws_appautoscaling_target.this.service_namespace
  resource_id        = aws_appautoscaling_target.this.resource_id
  scalable_dimension = aws_appautoscaling_target.this.scalable_dimension
  policy_type        = "TargetTrackingScaling"
  target_tracking_scaling_policy_configuration {
    target_value = 60
    predefined_metric_specification {
      predefined_metric_type = "ECSServiceAverageCPUUtilization"
    }
  }
}

# ---- アラーム ----

# 使える server がない(タスクが起動しない・ヘルスチェックに落ち続ける)
resource "aws_cloudwatch_metric_alarm" "no_healthy_task" {
  alarm_name          = "${var.name}-no-healthy-task"
  alarm_description   = "ALB の転送先に正常な server がない。ECS のイベントとタスクのログを見る"
  namespace           = "AWS/ApplicationELB"
  metric_name         = "HealthyHostCount"
  dimensions          = { LoadBalancer = var.alb_arn_suffix, TargetGroup = aws_lb_target_group.this.arn_suffix }
  statistic           = "Minimum"
  period              = 60
  evaluation_periods  = 2
  threshold           = 1
  comparison_operator = "LessThanThreshold"
  treat_missing_data  = "breaching"
  alarm_actions       = var.alarm_actions
  ok_actions          = var.alarm_actions
}

# server が 5xx を返している(ALB が届かずに返した 5xx は load-balancer のアラームで見る)。
# gRPC-Web のエラー(Internal・Unavailable など)は HTTP 200 で返るので、ここには数えられない(アプリのログで見る)
resource "aws_cloudwatch_metric_alarm" "http_5xx" {
  alarm_name          = "${var.name}-http-5xx"
  alarm_description   = "${var.name} の server が 5xx を返している。server のログを見る"
  namespace           = "AWS/ApplicationELB"
  metric_name         = "HTTPCode_Target_5XX_Count"
  dimensions          = { LoadBalancer = var.alb_arn_suffix, TargetGroup = aws_lb_target_group.this.arn_suffix }
  statistic           = "Sum"
  period              = 300
  evaluation_periods  = 1
  threshold           = 5
  comparison_operator = "GreaterThanOrEqualToThreshold"
  treat_missing_data  = "notBreaching"
  alarm_actions       = var.alarm_actions
  ok_actions          = var.alarm_actions
}
