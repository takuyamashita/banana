-- 出来事を記録したときのトレース(W3C traceparent)。relay が SQS のメッセージ属性に載せ、受け手がその続きにする
alter table outbox add column traceparent varchar(64) null;
