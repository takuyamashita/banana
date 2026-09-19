# サービスで共有する入口(ALB)と ECS クラスター。
# リスナーは、どのサービスにも当たらないパスには 404 を返す。サービスごとの振り分け(パス)は backend-service が足す。
# ブラウザからは同じ API のドメインで、RPC のパス(/acme.payroll.v1.* など)でサービスに届く
resource "aws_ecs_cluster" "this" {
  name = var.name
  # タスクごとの CPU・メモリの細かい指標。有料なので、見る必要がある環境だけで有効にする
  setting {
    name  = "containerInsights"
    value = var.container_insights ? "enabled" : "disabled"
  }
}

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

resource "aws_lb" "this" {
  name                       = var.name
  load_balancer_type         = "application"
  subnets                    = var.public_subnet_ids
  security_groups            = [aws_security_group.alb.id]
  drop_invalid_header_fields = true
  enable_deletion_protection = var.deletion_protection
}

resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.this.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  certificate_arn   = var.certificate_arn
  default_action {
    type = "fixed-response"
    fixed_response {
      content_type = "text/plain"
      message_body = "not found"
      status_code  = "404"
    }
  }
}

# ALB がサービスに届かずに 5xx を返している(どのサービスでも)。サービスが返した 5xx は backend-service のアラームで見る
resource "aws_cloudwatch_metric_alarm" "elb_5xx" {
  alarm_name          = "${var.name}-elb-5xx"
  alarm_description   = "ALB がサービスに届かず 5xx を返している。ALB のアクセスログと各サービスのタスクを見る"
  namespace           = "AWS/ApplicationELB"
  metric_name         = "HTTPCode_ELB_5XX_Count"
  dimensions          = { LoadBalancer = aws_lb.this.arn_suffix }
  statistic           = "Sum"
  period              = 300
  evaluation_periods  = 1
  threshold           = 5
  comparison_operator = "GreaterThanOrEqualToThreshold"
  treat_missing_data  = "notBreaching"
  alarm_actions       = var.alarm_actions
  ok_actions          = var.alarm_actions
}
