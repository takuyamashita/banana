use async_trait::async_trait;
use aws_sdk_cognitoidentityprovider::Client;
use aws_sdk_cognitoidentityprovider::operation::admin_create_user::AdminCreateUserError;
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
            .map_err(|err| {
                if err
                    .as_service_error()
                    .is_some_and(AdminCreateUserError::is_username_exists_exception)
                {
                    UserDirectoryError::AlreadyExists
                } else {
                    unavailable(err)
                }
            })?;

        // Cognito固有のレスポンスをdomainの型に変換するのはここの責務。
        // Username が sub と一致するのは user pool の UsernameAttributes に email を指定した場合だけなので、
        // sub 属性を直接読む
        let sub = out
            .user()
            .and_then(|u| u.attributes().iter().find(|a| a.name() == "sub"))
            .and_then(|a| a.value())
            .ok_or_else(|| UserDirectoryError::Unavailable("sub attribute missing".into()))?;
        UserId::parse(sub).map_err(|e| UserDirectoryError::Invalid(e.to_string()))
    }

    async fn disable_user(&self, id: &UserId) -> Result<(), UserDirectoryError> {
        // AdminDisableUser は Username を受け取るが、sub も Username の代わりに使える
        self.client
            .admin_disable_user()
            .user_pool_id(&self.user_pool_id)
            .username(id.as_str())
            .send()
            .await
            .map_err(unavailable)?;
        Ok(())
    }
}
