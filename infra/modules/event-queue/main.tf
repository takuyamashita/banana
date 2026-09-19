# 出来事の受け手ごとの FIFO キュー。受け手が要る出来事のトピックを購読し、要る種類(event_type)だけを受け取る。
# relay は MessageGroupId(集約)と MessageDeduplicationId(outbox.id)を付けるので、集約ごとの順序が保たれる
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
  # 既定の4日だと、受け手を止めたまま連休をまたぐと消える(outbox は送信済みなので戻せない)。最大の14日にする
  message_retention_seconds = 1209600
  sqs_managed_sse_enabled   = true
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

# 出来事のトピックを購読する。本文はそのまま(封筒の JSON)届け、メッセージ属性(event_type・traceparent)も渡す
resource "aws_sns_topic_subscription" "this" {
  count                = length(var.subscriptions)
  topic_arn            = var.subscriptions[count.index].topic_arn
  protocol             = "sqs"
  endpoint             = aws_sqs_queue.this.arn
  raw_message_delivery = true
  filter_policy        = jsonencode({ event_type = var.subscriptions[count.index].event_types })
}

# 購読したトピックだけが、このキューに送れる
data "aws_iam_policy_document" "from_topics" {
  count = length(var.subscriptions) > 0 ? 1 : 0
  statement {
    actions   = ["sqs:SendMessage"]
    resources = [aws_sqs_queue.this.arn]
    principals {
      type        = "Service"
      identifiers = ["sns.amazonaws.com"]
    }
    condition {
      test     = "ArnEquals"
      variable = "aws:SourceArn"
      values   = [for s in var.subscriptions : s.topic_arn]
    }
  }
}

resource "aws_sqs_queue_policy" "this" {
  count     = length(var.subscriptions) > 0 ? 1 : 0
  queue_url = aws_sqs_queue.this.id
  policy    = data.aws_iam_policy_document.from_topics[0].json
}
