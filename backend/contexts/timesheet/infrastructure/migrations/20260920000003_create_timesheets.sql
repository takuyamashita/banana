-- 勤務表。派遣社員1人の1か月分で、同じ派遣社員・同じ月は1つだけ
create table timesheets (
  id               bigint        not null auto_increment primary key,
  staff_id         bigint        not null,
  work_year        smallint unsigned not null,
  work_month       tinyint unsigned  not null,
  -- draft(作成中)・submitted(申告済み)・approved(承認済み)
  status           varchar(16)   not null,
  -- 差し戻されたときの理由。作成中の間だけ持つ
  returned_reason  varchar(500)  null,
  submitted_at     datetime(6)   null,
  approved_at      datetime(6)   null,
  created_at       datetime(6)   not null default current_timestamp(6),
  unique key uk_timesheets_staff_month (staff_id, work_year, work_month),
  index idx_timesheets_status (status, submitted_at),
  constraint fk_timesheets_staff foreign key (staff_id) references staff (id),
  constraint ck_timesheets_month check (work_year between 2000 and 2999 and work_month between 1 and 12),
  -- 状態と日時が食い違う行は記録させない(申告済みには申告日時、承認済みには承認日時が必ずある)
  constraint ck_timesheets_status check (
    (status = 'draft' and approved_at is null)
    or (status = 'submitted' and submitted_at is not null and approved_at is null and returned_reason is null)
    or (status = 'approved' and submitted_at is not null and approved_at is not null and returned_reason is null)
  )
);
