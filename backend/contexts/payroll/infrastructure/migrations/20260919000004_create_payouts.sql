-- 振込依頼。確定した給与明細1つにつき1件で、振込先が受け付けたか断ったかを残す
create table payouts (
  id             bigint        not null auto_increment primary key,
  payslip_id     bigint        not null,
  staff_id       bigint        not null,
  amount_yen     bigint        not null,
  -- accepted(受け付けられた)/ rejected(断られた)
  outcome        varchar(16)   not null,
  -- 受け付けられたときに振込先が発行した受付番号
  receipt        varchar(255)  null,
  -- 断られたときの理由
  reject_reason  varchar(1000) null,
  created_at     datetime(6)   not null default current_timestamp(6),
  unique key uk_payout_payslip (payslip_id)
);

-- 振込依頼の記録で二重の依頼を防ぐので、出来事ごとの処理済みの記録は使わない
drop table processed_events;
