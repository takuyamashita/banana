create table outbox (
  id              bigint       not null auto_increment primary key,
  aggregate_type  varchar(50)  not null,
  aggregate_id    bigint       not null,
  event_type      varchar(100) not null,
  payload         json         not null,
  created_at      datetime(6)  not null default current_timestamp(6),
  published_at    datetime(6)  null,
  -- id が採番順 = 発生順なので、未送信分を id 順で取ればよい
  index idx_unpublished (published_at, id)
);

-- consumer の冪等化。SQS は at-least-once なので処理済みイベントIDを記録する
create table processed_events (
  event_id      bigint      not null primary key,
  processed_at  datetime(6) not null default current_timestamp(6)
);
