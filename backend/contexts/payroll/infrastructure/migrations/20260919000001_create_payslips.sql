-- 給与明細。二重確定は「有効な版が1つ」を生成列 + ユニーク制約で DB 側でも保証する
create table payslips (
  id             bigint            not null auto_increment primary key,
  staff_id       bigint            not null,
  -- u16 / u8 で読み書きするので unsigned にする(signed だと sqlx は i16 / i8 に推論する)
  pay_year       smallint unsigned not null,
  pay_month      tinyint unsigned  not null,
  status         varchar(20)       not null,
  finalized_at   datetime(6)       null,
  finalized_by   bigint            null,
  superseded_at  datetime(6)       null,
  created_at     datetime(6)       not null default current_timestamp(6),

  -- 有効な版(superseded_at is null)だけを一意にする。
  -- MySQLは部分インデックスを張れないので、
  -- 無効な版ではNULLになる生成列を作って重複を許す形で表現する
  active_key varchar(32) generated always as (
    if(superseded_at is null, concat(staff_id, '-', pay_year, '-', pay_month), null)
  ) stored,
  unique key uk_active_payslip (active_key),

  index idx_staff_period (staff_id, pay_year, pay_month)
);

-- 明細行は payslip 集約の一部なので、同じ集約内の外部キーは張ってよい
create table payslip_lines (
  id            bigint       not null auto_increment primary key,
  payslip_id    bigint       not null,
  -- 案件は別集約なので外部キーは張らない
  project_id    bigint       not null,
  work_minutes  int unsigned not null,
  hourly_rate   bigint       not null,
  constraint fk_payslip_lines_payslip foreign key (payslip_id) references payslips (id)
);
