-- 既にある明細行の案件名を、今の案件名で埋める。
-- 案件の登録を確かめる前に作られた行は案件が見つからないので、案件番号で表す
update payslip_lines l
  left join projects p on p.id = l.project_id
  set l.project_name = coalesce(p.name, concat('案件 #', l.project_id))
  where l.project_name = '';
