# 集約間のイベント用 FIFO キュー。relay は MessageGroupId(集約ID)と MessageDeduplicationId(outbox.id)を付ける
resource "aws_sqs_queue" "dlq" {
  name                      = "${var.name}-dlq.fifo"
  fifo_queue                = true
  message_retention_seconds = 1209600
  sqs_managed_sse_enabled   = true
}

resource "aws_sqs_queue" "this" {
  name                        = "${var.name}.fifo"
  fifo_queue                  = true
  content_based_deduplication = false
  # 同じ集約(グループ)内の順序を保ったまま、グループ間は並列に処理できるようにする
  deduplication_scope   = "messageGroup"
  fifo_throughput_limit = "perMessageGroupId"
  # Lambda のタイムアウトの6倍以上にする(AWS の推奨)
  visibility_timeout_seconds = var.visibility_timeout_seconds
  sqs_managed_sse_enabled    = true
  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.dlq.arn
    maxReceiveCount     = 5
  })
}
