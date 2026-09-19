-- 派遣社員の写し。給与(payroll)の staff.registered で届く。番号は給与で振られたものをそのまま使う
create table staff (
  id          bigint       not null primary key,
  -- 認証基盤上のID(アクセストークンの sub)。ログインした本人がどの派遣社員かを知るのに使う
  user_id     varchar(64)  not null,
  name        varchar(100) not null,
  updated_at  datetime(6)  not null default current_timestamp(6) on update current_timestamp(6),
  unique key uk_staff_user_id (user_id)
);
