-- 勤務表の稼働の行。1日・1案件に1行
create table timesheet_entries (
  timesheet_id  bigint  not null,
  work_date     date    not null,
  project_id    bigint  not null,
  -- 15分単位、1日24時間まで(domain の WorkMinutes と同じ決まり)
  work_minutes  int     not null,
  primary key (timesheet_id, work_date, project_id),
  constraint fk_timesheet_entries_timesheet foreign key (timesheet_id) references timesheets (id),
  constraint fk_timesheet_entries_project foreign key (project_id) references projects (id),
  constraint ck_timesheet_entries_minutes check (work_minutes between 15 and 1440 and work_minutes % 15 = 0)
);
