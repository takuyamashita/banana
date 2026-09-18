create table staff (
  id            bigint       not null auto_increment primary key,
  -- 認証基盤上のID(Cognito の sub)。雇用記録の id とは別物
  user_id       varchar(64)  not null,
  email         varchar(254) not null,
  display_name  varchar(50)  not null,
  created_at    datetime(6)  not null default current_timestamp(6),
  unique key uk_staff_user_id (user_id),
  unique key uk_staff_email (email)
);

create table projects (
  id          bigint       not null auto_increment primary key,
  name        varchar(100) not null,
  created_at  datetime(6)  not null default current_timestamp(6)
);
