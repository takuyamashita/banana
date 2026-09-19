-- 案件の写し。給与(payroll)の project.created で届く。番号は給与で振られたものをそのまま使う
create table projects (
  id          bigint       not null primary key,
  name        varchar(100) not null,
  updated_at  datetime(6)  not null default current_timestamp(6) on update current_timestamp(6)
);
