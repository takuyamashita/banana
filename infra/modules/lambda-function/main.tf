# Rust の Lambda(cargo lambda build --arm64 の zip)。SQS トリガーは任意

resource "aws_cloudwatch_log_group" "this" {
  name              = "/aws/lambda/${var.name}"
  retention_in_days = 30
}

data "aws_iam_policy_document" "assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "this" {
  name               = var.name
  assume_role_policy = data.aws_iam_policy_document.assume.json
}

resource "aws_iam_role_policy_attachment" "vpc" {
  role       = aws_iam_role.this.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
}

resource "aws_iam_role_policy_attachment" "xray" {
  role       = aws_iam_role.this.name
  policy_arn = "arn:aws:iam::aws:policy/AWSXRayDaemonWriteAccess"
}

data "aws_iam_policy_document" "sqs" {
  count = var.sqs_queue_arn == null ? 0 : 1
  statement {
    actions   = ["sqs:ReceiveMessage", "sqs:DeleteMessage", "sqs:GetQueueAttributes", "sqs:ChangeMessageVisibility"]
    resources = [var.sqs_queue_arn]
  }
}

resource "aws_iam_role_policy" "sqs" {
  count  = var.sqs_queue_arn == null ? 0 : 1
  name   = "${var.name}-sqs"
  role   = aws_iam_role.this.id
  policy = data.aws_iam_policy_document.sqs[0].json
}

resource "aws_iam_role_policy" "extra" {
  count  = var.policy_json == null ? 0 : 1
  name   = "${var.name}-extra"
  role   = aws_iam_role.this.id
  policy = var.policy_json
}

resource "aws_security_group" "this" {
  name        = var.name
  description = "Lambda ${var.name}"
  vpc_id      = var.vpc_id
}

# 外向きは HTTPS(AWS API・外部API)と VPC 内の MySQL だけ
resource "aws_vpc_security_group_egress_rule" "https" {
  security_group_id = aws_security_group.this.id
  cidr_ipv4         = "0.0.0.0/0"
  ip_protocol       = "tcp"
  from_port         = 443
  to_port           = 443
  description       = "AWS APIs, external APIs"
}

resource "aws_vpc_security_group_egress_rule" "mysql" {
  security_group_id = aws_security_group.this.id
  cidr_ipv4         = var.vpc_cidr
  ip_protocol       = "tcp"
  from_port         = 3306
  to_port           = 3306
  description       = "MySQL in VPC"
}

resource "aws_lambda_function" "this" {
  function_name = var.name
  role          = aws_iam_role.this.arn
  s3_bucket     = var.artifact_bucket
  s3_key        = var.artifact_key
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["arm64"]
  memory_size   = var.memory_size
  timeout       = var.timeout
  # ADOT collector レイヤー。アプリは OTLP を localhost:4317 に出すだけ
  layers = var.adot_layer_arn == null ? [] : [var.adot_layer_arn]
  environment {
    variables = merge({ APP_ENV = var.env }, var.environment)
  }
  vpc_config {
    subnet_ids         = var.subnet_ids
    security_group_ids = [aws_security_group.this.id]
  }
  tracing_config {
    mode = "Active"
  }
  logging_config {
    log_format = "JSON"
    log_group  = aws_cloudwatch_log_group.this.name
  }
}

resource "aws_lambda_event_source_mapping" "sqs" {
  count            = var.sqs_queue_arn == null ? 0 : 1
  event_source_arn = var.sqs_queue_arn
  function_name    = aws_lambda_function.this.arn
  # FIFO キューでは最大 10
  batch_size = 10
  # 失敗した件だけを返してキューに戻す。関数は、失敗した件と同じグループの後ろの件も返す(順序を保つ)
  function_response_types = ["ReportBatchItemFailures"]
}
