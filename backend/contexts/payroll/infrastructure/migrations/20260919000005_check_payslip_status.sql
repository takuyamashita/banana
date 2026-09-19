-- 給与明細の状態は作成中か確定済みで、確定日時があるのは確定済みだけ(アプリの組み立て直しと同じ規則を DB でも守る)
alter table payslips
  add constraint ck_payslip_status check (status in ('draft', 'finalized')),
  add constraint ck_payslip_finalized_at check ((status = 'finalized') = (finalized_at is not null));
