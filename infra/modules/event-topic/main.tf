# サービスが出す出来事の SNS FIFO トピック。受け手ごとの SQS キュー(event-queue)が購読し、
# 購読のフィルターで要る種類(メッセージ属性 event_type)だけを受け取る。
# relay は MessageGroupId(集約)・MessageDeduplicationId(outbox.id)を付けるので、集約ごとの順序が保たれる
resource "aws_sns_topic" "this" {
  name                        = "${var.name}.fifo"
  fifo_topic                  = true
  content_based_deduplication = false
  kms_master_key_id           = "alias/aws/sns"
}
