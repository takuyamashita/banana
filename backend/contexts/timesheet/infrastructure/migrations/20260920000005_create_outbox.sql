-- 他のサービス(給与)に知らせる出来事。relay(platform-messaging)が記録した順に送る。
-- 形はどのサービスも同じ(platform-messaging の relay が読む)
create table outbox (
  id              bigint        not null auto_increment primary key,
  aggregate_type  varchar(50)   not null,
  aggregate_id    bigint        not null,
  event_type      varchar(100)  not null,
  payload         json          not null,
  created_at      datetime(6)   not null default current_timestamp(6),
  published_at    datetime(6)   null,
  -- relay が送れなかった回数と、最後のエラー
  attempts        int           not null default 0,
  last_error      varchar(1000) null,
  -- 出来事を記録したときのトレース(W3C traceparent)。受け手がその続きにする
  traceparent     varchar(64)   null,
  index idx_unpublished (published_at, id)
);
