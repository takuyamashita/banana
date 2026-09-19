-- relay が送れなかった回数と、最後のエラー。失敗し続ける出来事を見つけ、諦める回数を数えるために使う
alter table outbox
  add column attempts int not null default 0,
  add column last_error varchar(1000) null;
