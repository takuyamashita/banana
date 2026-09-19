-- サービスごとのデータベースとユーザー(初回の起動時に1回だけ流れる)。
-- ユーザーは自分のデータベースだけを触れる。AWS と同じく、別のサービスのデータは読めない。
-- ローカルではマイグレーションも同じユーザーで流すので、自分のデータベースの表は作り変えられる
create database payroll character set utf8mb4 collate utf8mb4_0900_ai_ci;
create user 'payroll'@'%' identified by 'payroll';
grant all on payroll.* to 'payroll'@'%';

create database timesheet character set utf8mb4 collate utf8mb4_0900_ai_ci;
create user 'timesheet'@'%' identified by 'timesheet';
grant all on timesheet.* to 'timesheet'@'%';
