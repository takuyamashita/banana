use async_trait::async_trait;
use aws_sdk_cognitoidentityprovider::Client;
use aws_sdk_cognitoidentityprovider::operation::admin_delete_user::AdminDeleteUserError;
use aws_sdk_cognitoidentityprovider::types::AttributeType;
use payroll_usecase::ports::user_directory::{UserDirectory, UserDirectoryError};
use platform_kernel::{Email, UserId};

/// stg/prd 用
pub struct CognitoUserDirectory {
    client: Client,
    user_pool_id: String,
}

impl CognitoUserDirectory {
    #[must_use]
    pub fn new(client: Client, user_pool_id: String) -> Self {
        Self { client, user_pool_id }
    }
}

fn unavailable(err: impl std::error::Error + Send + Sync + 'static) -> UserDirectoryError {
    UserDirectoryError::Unavailable(
        aws_sdk_cognitoidentityprovider::error::DisplayErrorContext(err).to_string(),
    )
}

#[async_trait]
impl UserDirectory for CognitoUserDirectory {
    async fn create_user(
        &self,
        email: &Email,
        temporary_password: &str,
    ) -> Result<UserId, UserDirectoryError> {
        let email_attr = AttributeType::builder()
            .name("email")
            .value(email.as_str())
            .build()
            .map_err(unavailable)?;
        let verified_attr = AttributeType::builder()
            .name("email_verified")
            .value("true")
            .build()
            .map_err(unavailable)?;

        let out = self
            .client
            .admin_create_user()
            .user_pool_id(&self.user_pool_id)
            .username(email.as_str())
            .temporary_password(temporary_password)
            .user_attributes(email_attr)
            .user_attributes(verified_attr)
            .send()
            .await
            .map_err(|err| match err.as_service_error() {
                Some(e) if e.is_username_exists_exception() => UserDirectoryError::AlreadyExists,
                // パスワードポリシー(12文字以上・大小英字・数字)に合わない
                Some(e) if e.is_invalid_password_exception() => UserDirectoryError::InvalidPassword,
                Some(e) if e.is_invalid_parameter_exception() => {
                    let detail =
                        aws_sdk_cognitoidentityprovider::error::DisplayErrorContext(e).to_string();
                    tracing::info!(detail, "cognito rejected the new user");
                    UserDirectoryError::Invalid { detail }
                }
                _ => unavailable(err),
            })?;

        // Cognito固有のレスポンスをdomainの型に変換するのはここの責務。
        // Username が sub と一致するのは user pool の UsernameAttributes に email を指定した場合だけなので、
        // sub 属性を直接読む
        let sub = out
            .user()
            .and_then(|u| u.attributes().iter().find(|a| a.name() == "sub"))
            .and_then(|a| a.value())
            .ok_or_else(|| UserDirectoryError::Unavailable("sub attribute missing".into()))?;
        let user_id =
            UserId::parse(sub).map_err(|e| UserDirectoryError::Unavailable(e.to_string()))?;

        // 派遣社員のロール(グループ staff)に入れる
        let added = self
            .client
            .admin_add_user_to_group()
            .user_pool_id(&self.user_pool_id)
            .username(user_id.as_str())
            .group_name("staff")
            .send()
            .await;
        if let Err(err) = added {
            let _ = self.delete_user(&user_id).await;
            return Err(unavailable(err));
        }
        Ok(user_id)
    }

    async fn delete_user(&self, id: &UserId) -> Result<(), UserDirectoryError> {
        // AdminDeleteUser は Username を受け取るが、sub も Username の代わりに使える
        let deleted = self
            .client
            .admin_delete_user()
            .user_pool_id(&self.user_pool_id)
            .username(id.as_str())
            .send()
            .await;
        match deleted {
            Ok(_) => Ok(()),
            // 既にないなら、消したのと同じ
            Err(err)
                if err
                    .as_service_error()
                    .is_some_and(AdminDeleteUserError::is_user_not_found_exception) =>
            {
                Ok(())
            }
            Err(err) => Err(unavailable(err)),
        }
    }
}
