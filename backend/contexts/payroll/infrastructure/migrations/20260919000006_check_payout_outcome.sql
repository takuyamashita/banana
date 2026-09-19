-- 振込依頼の答えは、受け付けられた(受付番号がある)か、断られた(理由がある)のどちらか
alter table payouts
  add constraint ck_payout_outcome check (
    (outcome = 'accepted' and receipt is not null and reject_reason is null)
    or (outcome = 'rejected' and receipt is null and reject_reason is not null)
  );
