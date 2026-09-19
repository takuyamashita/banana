//! 派遣社員の登録・一覧と、ログインした本人の派遣社員情報

use std::sync::Arc;

use payroll_domain::staff::{DisplayName, NewStaff, Staff, StaffId};
use platform_kernel::AuthenticatedUser;
use platform_kernel::Email;

use crate::UseCaseError;
use crate::ports::database::Database;
use crate::ports::events::{EventOutbox, PayrollEvent};
use crate::ports::queries::{StaffQuery, StaffView};
use crate::ports::repository::StaffRepository;
use crate::ports::user_directory::UserDirectory;

/// 派遣社員の登録で管理者が入力する内容
pub struct CreateStaffInput {
    /// ログインと連絡に使うメールアドレス
    pub email: Email,
    /// 表示名
    pub display_name: DisplayName,
    /// 初回ログイン用の仮パスワード。本人が初回ログインで変更する
    pub temporary_password: String,
}

/// 管理者が派遣社員を登録する。
///
/// 派遣社員のログイン用アカウントを発行し、そのアカウントと雇用記録を結びつける。
/// 同じメールアドレスの派遣社員は登録できない。登録したことは他の業務(勤怠など)に知らせる
pub struct CreateStaffUseCase {
    staff_repository: Arc<dyn StaffRepository>,
    outbox: Arc<dyn EventOutbox>,
    db: Arc<dyn Database>,
    user_directory: Arc<dyn UserDirectory>,
}

impl CreateStaffUseCase {
    #[must_use]
    pub fn new(
        staff_repository: Arc<dyn StaffRepository>,
        outbox: Arc<dyn EventOutbox>,
        db: Arc<dyn Database>,
        user_directory: Arc<dyn UserDirectory>,
    ) -> Self {
        Self { staff_repository, outbox, db, user_directory }
    }

    /// 派遣社員を登録し、振られた派遣社員番号を返す。
    ///
    /// 同じメールアドレスの派遣社員がいれば `Conflict` になる。アカウントを発行した後で登録に
    /// 失敗したときは、発行したアカウントを消してから失敗を返す(登録し直せるようにする)
    pub async fn execute(&self, input: CreateStaffInput) -> Result<StaffId, UseCaseError> {
        if self.staff_repository.find_by_email(&input.email).await?.is_some() {
            return Err(UseCaseError::Conflict("同じメールアドレスの派遣社員がいます".into()));
        }

        let user_id =
            self.user_directory.create_user(&input.email, &input.temporary_password).await?;

        let new = NewStaff::new(user_id.clone(), input.email, input.display_name);
        match self.register(&new).await {
            Ok(id) => Ok(id),
            Err(err) => match self.user_directory.delete_user(&user_id).await {
                Ok(()) => Err(err),
                // 消せなかったアカウントは誰の派遣社員とも結びつかずに残るので、管理者が消す必要がある
                Err(cleanup) => Err(UseCaseError::Internal(format!(
                    "派遣社員の登録に失敗し({err})、発行したアカウント {} の削除にも失敗しました({cleanup})",
                    user_id.as_str()
                ))),
            },
        }
    }

    /// 雇用記録と、登録したという出来事を一緒に記録する(どちらか一方だけが残ることはない)
    async fn register(&self, new: &NewStaff) -> Result<StaffId, UseCaseError> {
        let mut tx = self.db.transaction().await?;
        let id = self.staff_repository.insert(&mut tx, new).await?;
        self.outbox.append(&mut tx, PayrollEvent::Staff(new.registered_as(id))).await?;
        tx.commit().await?;
        Ok(id)
    }
}

/// 管理者が、登録済みの派遣社員を一覧する
pub struct ListStaffUseCase {
    query: Arc<dyn StaffQuery>,
}

impl ListStaffUseCase {
    #[must_use]
    pub fn new(query: Arc<dyn StaffQuery>) -> Self {
        Self { query }
    }

    pub async fn execute(&self) -> Result<Vec<StaffView>, UseCaseError> {
        Ok(self.query.list().await?)
    }
}

/// ログイン中の利用者が、自分がどの派遣社員かを知る。
/// 派遣社員として登録されていない利用者(管理者など)は `None` になる
pub struct GetMeUseCase {
    staff_repository: Arc<dyn StaffRepository>,
}

impl GetMeUseCase {
    #[must_use]
    pub fn new(staff_repository: Arc<dyn StaffRepository>) -> Self {
        Self { staff_repository }
    }

    pub async fn execute(&self, user: &AuthenticatedUser) -> Result<Option<Staff>, UseCaseError> {
        Ok(self.staff_repository.find_by_user_id(&user.user_id).await?)
    }
}
