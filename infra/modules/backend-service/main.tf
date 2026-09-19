# gRPC(+ gRPC-Web)サーバーを ECS Fargate で動かす。ALB → server:50051。
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

resource "aws_cloudwatch_log_group" "this" {
  name              = "/ecs/${var.name}"
  retention_in_days = 30
}

resource "aws_ecs_cluster" "this" {
  name = var.name
  setting {
    name  = "containerInsights"
    value = "enabled"
  }
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
  # 起動時に DB 接続文字列を読む
  statement {
    actions   = ["secretsmanager:GetSecretValue"]
    resources = [var.database_url_secret_arn]
  }
  # outbox relay がイベントを送る
  statement {
    actions   = ["sqs:SendMessage", "sqs:GetQueueAttributes"]
    resources = [var.queue_arn]
  }
  # UserDirectory(Cognito)で派遣社員のユーザーを作る・無効化する
  statement {
    actions   = ["cognito-idp:AdminCreateUser", "cognito-idp:AdminAddUserToGroup", "cognito-idp:AdminDeleteUser"]
    resources = [var.user_pool_arn]
  }
  # ADOT collector がトレースを X-Ray に送る
  statement {
    actions   = ["xray:PutTraceSegments", "xray:PutTelemetryRecords", "xray:GetSamplingRules", "xray:GetSamplingTargets"]
    resources = ["*"]
  }
}

resource "aws_iam_role_policy" "task" {
  name   = "${var.name}-task"
  role   = aws_iam_role.task.id
  policy = data.aws_iam_policy_document.task.json
}

# ---- ネットワーク ----

resource "aws_security_group" "alb" {
  name        = "${var.name}-alb"
  description = "ALB for ${var.name}"
  vpc_id      = var.vpc_id
}

resource "aws_vpc_security_group_ingress_rule" "alb_https" {
  security_group_id = aws_security_group.alb.id
  cidr_ipv4         = "0.0.0.0/0"
  ip_protocol       = "tcp"
  from_port         = 443
  to_port           = 443
  description       = "HTTPS from internet"
}

resource "aws_vpc_security_group_egress_rule" "alb_to_task" {
  security_group_id            = aws_security_group.alb.id
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
  referenced_security_group_id = aws_security_group.alb.id
  ip_protocol                  = "tcp"
  from_port                    = 50051
  to_port                      = 50051
  description                  = "From ALB"
}

# 外向きは HTTPS(AWS API・JWKS・振込API)と VPC 内の MySQL だけ
resource "aws_vpc_security_group_egress_rule" "task_https" {
  security_group_id = aws_security_group.task.id
  cidr_ipv4         = "0.0.0.0/0"
  ip_protocol       = "tcp"
  from_port         = 443
  to_port           = 443
  description       = "AWS APIs, JWKS"
}

resource "aws_vpc_security_group_egress_rule" "task_mysql" {
  security_group_id = aws_security_group.task.id
  cidr_ipv4         = var.vpc_cidr
  ip_protocol       = "tcp"
  from_port         = 3306
  to_port           = 3306
  description       = "MySQL in VPC"
}

resource "aws_lb" "this" {
  name                       = var.name
  load_balancer_type         = "application"
  subnets                    = var.public_subnet_ids
  security_groups            = [aws_security_group.alb.id]
  drop_invalid_header_fields = true
  enable_deletion_protection = var.deletion_protection
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

resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.this.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  certificate_arn   = var.certificate_arn
  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.this.arn
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
  task_role_arn            = aws_iam_role.task.arn
  runtime_platform {
    operating_system_family = "LINUX"
    cpu_architecture        = "ARM64"
  }
  container_definitions = jsonencode([
    {
      name             = "migrate"
      image            = var.image
      essential        = true
      entryPoint       = ["/app/migrate"]
      environment      = concat(local.environment, [{ name = "APP__TELEMETRY__OTLP_ENDPOINT", value = "" }])
      logConfiguration = local.log_configuration
    },
  ])
}

resource "aws_ecs_service" "this" {
  name                               = "${var.name}-server"
  cluster                            = aws_ecs_cluster.this.id
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
  depends_on = [aws_lb_listener.https]
}
