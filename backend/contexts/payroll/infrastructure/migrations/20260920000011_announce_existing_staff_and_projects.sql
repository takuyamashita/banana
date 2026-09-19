-- 出来事を記録するようになる前に登録された案件と派遣社員を、他のサービス(勤怠)に知らせる。
-- relay が他の出来事と同じように送る。ペイロードは proto(acme.payroll.events.v1)の JSON 形(int64 は文字列)。
-- 受け手は同じ出来事を何度受けても結果が変わらないので、送り直しになっても困らない
insert into outbox (aggregate_type, aggregate_id, event_type, payload)
select 'project', id, 'project.created',
       json_object('projectId', cast(id as char), 'name', name)
from projects
order by id;

insert into outbox (aggregate_type, aggregate_id, event_type, payload)
select 'staff', id, 'staff.registered',
       json_object('staffId', cast(id as char), 'userId', user_id, 'displayName', display_name)
from staff
order by id;
