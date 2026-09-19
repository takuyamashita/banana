-- 明細行に、給与明細を作った時点の案件名を残す(後で案件名が変わっても、作った明細の表記は変えない)。
--
-- 既定値の '' は切り替えの間だけのもの。migrate は新しい版の server より先に流れるので、
-- その間も古い版が案件名なしで明細行を書く。次の版で、残った '' を埋め直してから既定値を外す
alter table payslip_lines
  add column project_name varchar(100) not null default '';
