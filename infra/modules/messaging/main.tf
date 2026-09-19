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

# DLQ に1件でも入ったら知らせる。入るのは、再試行しても処理できなかった出来事(振込先につながらない状態が続いたなど)
resource "aws_cloudwatch_metric_alarm" "dlq_not_empty" {
  alarm_name          = "${var.name}-dlq-not-empty"
  alarm_description   = "${aws_sqs_queue.dlq.name} に処理できなかったメッセージがある。原因を直してから元のキューへ戻す(redrive)"
  namespace           = "AWS/SQS"
  metric_name         = "ApproximateNumberOfMessagesVisible"
  dimensions          = { QueueName = aws_sqs_queue.dlq.name }
  statistic           = "Maximum"
  period              = 300
  evaluation_periods  = 1
  threshold           = 0
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"
  alarm_actions       = var.alarm_actions
  ok_actions          = var.alarm_actions
}
