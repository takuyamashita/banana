use std::collections::{BTreeMap, HashSet};

use platform_kernel::Unsaved;
use time::OffsetDateTime;

use super::work_minutes::MINUTES_PER_DAY;
use super::{
    ProjectWork, ReturnReason, TimesheetError, TimesheetEvent, TimesheetId, WorkEntry, WorkMonth,
};
use crate::staff::StaffId;

/// 勤務表の状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimesheetStatus {
    /// 作成中。派遣社員が稼働を書いている(差し戻された勤務表も、直している間はここに戻る)
    Draft,
    /// 申告済み。派遣社員が書き終え、管理者の承認を待っている。書き直せない
    Submitted,
    /// 承認済み。その月の稼働が決まり、給与の計算の元になる。以後は変えられない
    Approved,
}

/// 1つの勤務表の稼働の行の上限。1か月・1人の稼働として十分な数
const MAX_ENTRIES: usize = 200;

/// 勤務表の内容。どの状態でも持つ情報
#[derive(Debug, Clone)]
pub struct TimesheetContent<Id = TimesheetId> {
    /// 勤務表番号
    id: Id,
    /// 勤務表を書く派遣社員
    staff_id: StaffId,
    /// 対象月
    month: WorkMonth,
    /// 日ごと・案件ごとの稼働。日付と案件の順
    entries: Vec<WorkEntry>,
}

impl<Id> TimesheetContent<Id> {
    /// 稼働の行は、対象月の日付だけ。同じ日の同じ案件は1行にまとめ、1日の合計は24時間まで
    fn new(
        id: Id,
        staff_id: StaffId,
        month: WorkMonth,
        mut entries: Vec<WorkEntry>,
    ) -> Result<Self, TimesheetError> {
        if entries.len() > MAX_ENTRIES {
            return Err(TimesheetError::TooManyEntries { max: MAX_ENTRIES });
        }
        let mut seen = HashSet::new();
        let mut per_day = BTreeMap::new();
        for entry in &entries {
            let date = entry.date();
            if !month.contains(date) {
                return Err(TimesheetError::OutsideMonth { date: date.to_string() });
            }
            if !seen.insert((date, entry.project_id())) {
                return Err(TimesheetError::DuplicateEntry { date: date.to_string() });
            }
            let day: &mut u32 = per_day.entry(date).or_default();
            *day += entry.minutes().as_minutes();
            if *day > MINUTES_PER_DAY {
                return Err(TimesheetError::DayOverflow { date: date.to_string() });
            }
        }
        entries.sort_by_key(|e| (e.date(), e.project_id()));
        Ok(Self { id, staff_id, month, entries })
    }

    #[must_use]
    pub fn staff_id(&self) -> StaffId {
        self.staff_id
    }

    #[must_use]
    pub fn month(&self) -> WorkMonth {
        self.month
    }

    #[must_use]
    pub fn entries(&self) -> &[WorkEntry] {
        &self.entries
    }

    /// 1か月の稼働の合計(分)
    #[must_use]
    pub fn total_minutes(&self) -> u32 {
        self.entries.iter().map(|e| e.minutes().as_minutes()).sum()
    }

    /// 案件ごとの稼働の合計。案件番号の順
    #[must_use]
    pub fn work_by_project(&self) -> Vec<ProjectWork> {
        let mut totals = BTreeMap::new();
        for entry in &self.entries {
            *totals.entry(entry.project_id()).or_default() += entry.minutes().as_minutes();
        }
        totals
            .into_iter()
            .map(|(project_id, minutes)| ProjectWork { project_id, minutes })
            .collect()
    }
}

impl TimesheetContent<TimesheetId> {
    #[must_use]
    pub fn id(&self) -> TimesheetId {
        self.id
    }
}

/// 作成中の勤務表。派遣社員が稼働を書いている
#[derive(Debug, Clone)]
pub struct DraftTimesheet<Id = TimesheetId> {
    content: TimesheetContent<Id>,
    /// 差し戻されたときの理由。直してもう一度申告するまで残る
    returned: Option<ReturnReason>,
}

/// まだ登録していない勤務表。登録するまでは作成中で、申告もできない
pub type NewTimesheet = DraftTimesheet<Unsaved>;

impl<Id> DraftTimesheet<Id> {
    #[must_use]
    pub fn content(&self) -> &TimesheetContent<Id> {
        &self.content
    }

    /// 差し戻されていれば、その理由
    #[must_use]
    pub fn returned_reason(&self) -> Option<&ReturnReason> {
        self.returned.as_ref()
    }

    /// 稼働を書き直す(書いてあった行は、渡した行で置き換わる)。差し戻しの理由は、申告するまで残す
    pub fn record(self, entries: Vec<WorkEntry>) -> Result<Self, TimesheetError> {
        let TimesheetContent { id, staff_id, month, .. } = self.content;
        Ok(Self {
            content: TimesheetContent::new(id, staff_id, month, entries)?,
            returned: self.returned,
        })
    }
}

impl DraftTimesheet<TimesheetId> {
    /// 書き終えた勤務表を `submitted_at` に申告する。稼働が1件もなければ申告できない。
    /// 申告できるのは登録済みの勤務表だけ
    pub fn submit(
        self,
        submitted_at: OffsetDateTime,
    ) -> Result<SubmittedTimesheet, TimesheetError> {
        if self.content.entries.is_empty() {
            return Err(TimesheetError::NothingToSubmit);
        }
        Ok(SubmittedTimesheet { content: self.content, submitted_at })
    }
}

/// 申告済みの勤務表。管理者の承認を待っている
#[derive(Debug, Clone)]
pub struct SubmittedTimesheet {
    content: TimesheetContent,
    /// 申告した日時
    submitted_at: OffsetDateTime,
}

impl SubmittedTimesheet {
    #[must_use]
    pub fn content(&self) -> &TimesheetContent {
        &self.content
    }

    #[must_use]
    pub fn submitted_at(&self) -> OffsetDateTime {
        self.submitted_at
    }

    /// 勤務表を `approved_at` に承認し、その月の稼働を決める。
    /// 承認した勤務表と、承認したという出来事(案件ごとの稼働の合計)を返す
    pub fn approve(self, approved_at: OffsetDateTime) -> (ApprovedTimesheet, TimesheetEvent) {
        let event = TimesheetEvent::Approved {
            timesheet_id: self.content.id,
            staff_id: self.content.staff_id,
            month: self.content.month,
            work: self.content.work_by_project(),
            approved_at,
        };
        (
            ApprovedTimesheet {
                content: self.content,
                submitted_at: self.submitted_at,
                approved_at,
            },
            event,
        )
    }

    /// 理由を付けて差し戻す。勤務表は作成中に戻り、派遣社員が直してもう一度申告する
    #[must_use]
    pub fn send_back(self, reason: ReturnReason) -> DraftTimesheet {
        DraftTimesheet { content: self.content, returned: Some(reason) }
    }
}

/// 承認済みの勤務表。その月の稼働が決まり、以後は変えられない
#[derive(Debug, Clone)]
pub struct ApprovedTimesheet {
    content: TimesheetContent,
    submitted_at: OffsetDateTime,
    /// 承認した日時
    approved_at: OffsetDateTime,
}

impl ApprovedTimesheet {
    #[must_use]
    pub fn content(&self) -> &TimesheetContent {
        &self.content
    }

    #[must_use]
    pub fn submitted_at(&self) -> OffsetDateTime {
        self.submitted_at
    }

    #[must_use]
    pub fn approved_at(&self) -> OffsetDateTime {
        self.approved_at
    }
}

/// 勤務表。派遣社員1人の、ある1か月分の稼働。
///
/// 同じ派遣社員・同じ月の勤務表は1つだけ存在する
#[derive(Debug, Clone)]
pub enum Timesheet {
    Draft(DraftTimesheet),
    Submitted(SubmittedTimesheet),
    Approved(ApprovedTimesheet),
}

impl Timesheet {
    /// 派遣社員がその月の勤務表を書き始める。はじめは稼働が1件もない作成中の勤務表
    #[must_use]
    pub fn start(staff_id: StaffId, month: WorkMonth) -> NewTimesheet {
        DraftTimesheet {
            content: TimesheetContent { id: Unsaved, staff_id, month, entries: Vec::new() },
            returned: None,
        }
    }

    #[must_use]
    pub fn content(&self) -> &TimesheetContent {
        match self {
            Self::Draft(t) => t.content(),
            Self::Submitted(t) => t.content(),
            Self::Approved(t) => t.content(),
        }
    }

    #[must_use]
    pub fn status(&self) -> TimesheetStatus {
        match self {
            Self::Draft(_) => TimesheetStatus::Draft,
            Self::Submitted(_) => TimesheetStatus::Submitted,
            Self::Approved(_) => TimesheetStatus::Approved,
        }
    }

    /// 登録済みの作成中の勤務表を、記録されている内容から組み立て直す
    pub fn reconstruct_draft(
        id: TimesheetId,
        staff_id: StaffId,
        month: WorkMonth,
        entries: Vec<WorkEntry>,
        returned: Option<ReturnReason>,
    ) -> Result<Self, TimesheetError> {
        let content = TimesheetContent::new(id, staff_id, month, entries)?;
        Ok(Self::Draft(DraftTimesheet { content, returned }))
    }

    /// 登録済みの申告済みの勤務表を、記録されている内容から組み立て直す
    pub fn reconstruct_submitted(
        id: TimesheetId,
        staff_id: StaffId,
        month: WorkMonth,
        entries: Vec<WorkEntry>,
        submitted_at: OffsetDateTime,
    ) -> Result<Self, TimesheetError> {
        let content = submitted_content(id, staff_id, month, entries)?;
        Ok(Self::Submitted(SubmittedTimesheet { content, submitted_at }))
    }

    /// 登録済みの承認済みの勤務表を、記録されている内容から組み立て直す
    pub fn reconstruct_approved(
        id: TimesheetId,
        staff_id: StaffId,
        month: WorkMonth,
        entries: Vec<WorkEntry>,
        submitted_at: OffsetDateTime,
        approved_at: OffsetDateTime,
    ) -> Result<Self, TimesheetError> {
        let content = submitted_content(id, staff_id, month, entries)?;
        Ok(Self::Approved(ApprovedTimesheet { content, submitted_at, approved_at }))
    }
}

/// 申告した勤務表の内容。申告した勤務表には稼働が必ずある
fn submitted_content(
    id: TimesheetId,
    staff_id: StaffId,
    month: WorkMonth,
    entries: Vec<WorkEntry>,
) -> Result<TimesheetContent, TimesheetError> {
    if entries.is_empty() {
        return Err(TimesheetError::NothingToSubmit);
    }
    TimesheetContent::new(id, staff_id, month, entries)
}

impl From<DraftTimesheet> for Timesheet {
    fn from(timesheet: DraftTimesheet) -> Self {
        Self::Draft(timesheet)
    }
}

impl From<SubmittedTimesheet> for Timesheet {
    fn from(timesheet: SubmittedTimesheet) -> Self {
        Self::Submitted(timesheet)
    }
}

impl From<ApprovedTimesheet> for Timesheet {
    fn from(timesheet: ApprovedTimesheet) -> Self {
        Self::Approved(timesheet)
    }
}

#[cfg(test)]
mod tests {
    use time::macros::{date, datetime};

    use super::*;
    use crate::project::ProjectId;
    use crate::timesheet::WorkMinutes;

    fn entry(date: time::Date, project: i64, minutes: u32) -> WorkEntry {
        WorkEntry::new(
            date,
            ProjectId::from_i64(project).unwrap(),
            WorkMinutes::from_minutes(minutes).unwrap(),
        )
    }

    fn staff() -> StaffId {
        StaffId::from_i64(3).unwrap()
    }

    fn september() -> WorkMonth {
        WorkMonth::new(2026, 9).unwrap()
    }

    #[allow(clippy::disallowed_methods, reason = "登録済みの作成中を作るため")]
    fn registered(entries: Vec<WorkEntry>) -> DraftTimesheet {
        let id = TimesheetId::from_i64(7).unwrap();
        let Timesheet::Draft(draft) =
            Timesheet::reconstruct_draft(id, staff(), september(), entries, None).unwrap()
        else {
            unreachable!()
        };
        draft
    }

    #[test]
    fn a_new_timesheet_starts_empty_as_a_draft() {
        let new = Timesheet::start(staff(), september());
        assert!(new.content().entries().is_empty());
        assert_eq!(new.returned_reason(), None);
    }

    #[test]
    fn only_days_of_the_month_can_be_recorded() {
        let err = Timesheet::start(staff(), september())
            .record(vec![entry(date!(2026 - 10 - 01), 1, 480)])
            .unwrap_err();
        assert_eq!(err, TimesheetError::OutsideMonth { date: "2026-10-01".into() });
    }

    #[test]
    fn the_same_project_on_the_same_day_is_one_row() {
        let err = Timesheet::start(staff(), september())
            .record(vec![
                entry(date!(2026 - 09 - 01), 1, 240),
                entry(date!(2026 - 09 - 01), 1, 240),
            ])
            .unwrap_err();
        assert_eq!(err, TimesheetError::DuplicateEntry { date: "2026-09-01".into() });
    }

    #[test]
    fn a_day_cannot_have_more_than_twenty_four_hours_across_projects() {
        // 1行ずつは上限以内でも、同じ日の合計が24時間を超えればありえない
        let err = Timesheet::start(staff(), september())
            .record(vec![
                entry(date!(2026 - 09 - 01), 1, 720),
                entry(date!(2026 - 09 - 01), 2, 735),
            ])
            .unwrap_err();
        assert_eq!(err, TimesheetError::DayOverflow { date: "2026-09-01".into() });
    }

    #[test]
    fn too_many_rows_are_rejected() {
        let rows = (1..=201).map(|p| entry(date!(2026 - 09 - 01), p, 15)).collect();
        assert_eq!(
            Timesheet::start(staff(), september()).record(rows).unwrap_err(),
            TimesheetError::TooManyEntries { max: 200 }
        );
    }

    #[test]
    fn rows_are_kept_in_order_of_day_and_project() {
        let draft = Timesheet::start(staff(), september())
            .record(vec![
                entry(date!(2026 - 09 - 02), 1, 60),
                entry(date!(2026 - 09 - 01), 2, 60),
                entry(date!(2026 - 09 - 01), 1, 60),
            ])
            .unwrap();
        let order: Vec<_> = draft
            .content()
            .entries()
            .iter()
            .map(|e| (e.date().day(), e.project_id().as_i64()))
            .collect();
        assert_eq!(order, [(1, 1), (1, 2), (2, 1)]);
    }

    #[test]
    fn an_empty_timesheet_cannot_be_submitted() {
        let err = registered(vec![]).submit(datetime!(2026-09-30 09:00 UTC)).unwrap_err();
        assert_eq!(err, TimesheetError::NothingToSubmit);
    }

    #[test]
    fn approval_decides_the_work_of_each_project_for_the_month() {
        let submitted = registered(vec![
            entry(date!(2026 - 09 - 01), 2, 480),
            entry(date!(2026 - 09 - 01), 1, 60),
            entry(date!(2026 - 09 - 02), 2, 450),
        ])
        .submit(datetime!(2026-09-30 09:00 UTC))
        .unwrap();
        let at = datetime!(2026-10-01 10:00 UTC);

        let (approved, event) = submitted.approve(at);

        assert_eq!(
            event,
            TimesheetEvent::Approved {
                timesheet_id: TimesheetId::from_i64(7).unwrap(),
                staff_id: staff(),
                month: september(),
                work: vec![
                    ProjectWork { project_id: ProjectId::from_i64(1).unwrap(), minutes: 60 },
                    ProjectWork { project_id: ProjectId::from_i64(2).unwrap(), minutes: 930 },
                ],
                approved_at: at,
            }
        );
        assert_eq!(approved.approved_at(), at);
        assert_eq!(approved.content().total_minutes(), 990);
        assert_eq!(Timesheet::from(approved).status(), TimesheetStatus::Approved);
    }

    #[test]
    fn a_returned_timesheet_goes_back_to_draft_with_the_reason_until_resubmitted() {
        let submitted = registered(vec![entry(date!(2026 - 09 - 01), 1, 480)])
            .submit(datetime!(2026-09-30 09:00 UTC))
            .unwrap();
        let reason = ReturnReason::new("9/1 の案件が違います").unwrap();

        let draft = submitted.send_back(reason.clone());
        assert_eq!(draft.returned_reason(), Some(&reason));

        // 直している間は理由が見える。申告し直すと、申告済みの勤務表には残らない
        let fixed = draft.record(vec![entry(date!(2026 - 09 - 01), 2, 480)]).unwrap();
        assert_eq!(fixed.returned_reason(), Some(&reason));
        let resubmitted = fixed.submit(datetime!(2026-09-30 12:00 UTC)).unwrap();
        assert_eq!(Timesheet::from(resubmitted).status(), TimesheetStatus::Submitted);
    }

    #[test]
    #[allow(clippy::disallowed_methods, reason = "記録から組み立て直すときの検証を確かめる")]
    fn a_submitted_timesheet_always_has_work() {
        let id = TimesheetId::from_i64(1).unwrap();
        assert_eq!(
            Timesheet::reconstruct_submitted(
                id,
                staff(),
                september(),
                vec![],
                datetime!(2026-09-30 09:00 UTC)
            )
            .unwrap_err(),
            TimesheetError::NothingToSubmit
        );
    }

    #[test]
    fn a_return_reason_must_say_something() {
        assert_eq!(ReturnReason::new("  "), Err(TimesheetError::InvalidReturnReason));
        assert_eq!(ReturnReason::new("あ".repeat(501)), Err(TimesheetError::InvalidReturnReason));
    }
}
