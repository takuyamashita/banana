-- 勤怠(timesheet)で承認された稼働の写し。timesheet.approved で届き、給与明細の明細行の稼働の元になる。
-- 同じ派遣社員・同じ月は、届いた内容で置き換える
create table approved_work (
  staff_id      bigint            not null,
  work_year     smallint unsigned not null,
  work_month    tinyint unsigned  not null,
  project_id    bigint            not null,
  work_minutes  int unsigned      not null,
  primary key (staff_id, work_year, work_month, project_id),
  constraint fk_approved_work_staff foreign key (staff_id) references staff (id),
  constraint fk_approved_work_project foreign key (project_id) references projects (id)
);
