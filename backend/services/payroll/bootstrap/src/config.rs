use platform_service::config::{
    AuthConfig, AuthProvider, DatabaseConfig, EmbeddedConfig, MessagingConfig, Required, Secret,
    SecretsConfig, ServerConfig, TelemetryConfig,
};
use serde::Deserialize;

/// 給与サービスの設定(config/payroll/)。環境ごとの設定もバイナリに埋め込む
const EMBEDDED: EmbeddedConfig = EmbeddedConfig {
    env_prefix: "PAYROLL",
    default: include_str!("../../../../../config/payroll/default.toml"),
    envs: &[
        ("local", include_str!("../../../../../config/payroll/local.toml")),
        ("dev", include_str!("../../../../../config/payroll/dev.toml")),
        ("stg", include_str!("../../../../../config/payroll/stg.toml")),
        ("prd", include_str!("../../../../../config/payroll/prd.toml")),
    ],
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PayoutProvider {
    Logging,
    Bank,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub env: String,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub user_directory: UserDirectoryConfig,
    pub messaging: MessagingConfig,
    pub payout: PayoutConfig,
    pub secrets: SecretsConfig,
    pub telemetry: TelemetryConfig,
}

/// 派遣社員のログインの作成・無効化(UserDirectory)。どちらの認証基盤かは auth.provider で決まる
#[derive(Debug, Deserialize)]
pub struct UserDirectoryConfig {
    pub cognito_user_pool_id: String,
    pub keycloak_base_url: String,
    pub keycloak_realm: String,
    pub keycloak_admin_client_id: String,
    pub keycloak_admin_client_secret: Secret,
}

#[derive(Debug, Deserialize)]
pub struct PayoutConfig {
    pub provider: PayoutProvider,
    pub base_url: String,
    pub api_key: Secret,
    /// 指定すると振込 API のキーを Secrets Manager から取る
    pub api_key_secret_id: String,
    /// 振込の依頼を受けるキュー(ローカルのポーラーだけが使う)
    pub queue_url: String,
}

/// `APP_ENV`(local/dev/stg/prd)を見て、埋め込みの default.toml に `{env}.toml` と
/// 環境変数 `PAYROLL__SECTION__KEY` を重ねる
pub fn load_config() -> anyhow::Result<AppConfig> {
    platform_service::config::load(&EMBEDDED)
}

impl AppConfig {
    /// server を動かすのに要る値がそろっているかを確かめる。足りなければ起動時に止める
    pub fn validate_for_server(&self) -> anyhow::Result<()> {
        let mut required = Required::default();
        required.server(&self.server, &self.database, &self.secrets, &self.auth, &self.messaging);
        required.value("messaging.inbox_queue_url", &self.messaging.inbox_queue_url);
        let directory = &self.user_directory;
        match self.auth.provider {
            AuthProvider::Cognito => {
                required
                    .value("user_directory.cognito_user_pool_id", &directory.cognito_user_pool_id);
            }
            AuthProvider::Keycloak => {
                required
                    .value("user_directory.keycloak_base_url", &directory.keycloak_base_url)
                    .value("user_directory.keycloak_realm", &directory.keycloak_realm)
                    .value(
                        "user_directory.keycloak_admin_client_id",
                        &directory.keycloak_admin_client_id,
                    )
                    .value(
                        "user_directory.keycloak_admin_client_secret",
                        directory.keycloak_admin_client_secret.expose(),
                    );
            }
        }
        required.check()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(env_toml: &str) -> AppConfig {
        config::Config::builder()
            .add_source(config::File::from_str(EMBEDDED.default, config::FileFormat::Toml))
            .add_source(config::File::from_str(env_toml, config::FileFormat::Toml))
            .set_override("env", "test")
            .unwrap()
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }

    fn env_toml(name: &str) -> &'static str {
        EMBEDDED.envs.iter().find(|(env, _)| *env == name).unwrap().1
    }

    #[test]
    fn the_local_config_is_complete() {
        config(env_toml("local")).validate_for_server().unwrap();
    }

    #[test]
    fn missing_or_placeholder_values_stop_the_server() {
        // AWS の環境の toml だけでは、Terraform が渡す値(issuer・トピックなど)が足りない
        let dev = env_toml("dev");
        let err = config(dev).validate_for_server().unwrap_err().to_string();
        assert!(err.contains("auth.issuer") && err.contains("messaging.topic_arn"), "{err}");

        let placeholder =
            format!("{dev}\n[messaging]\ninbox_queue_url = \"https://sqs/REPLACE_ME\"\n");
        let err = config(&placeholder).validate_for_server().unwrap_err().to_string();
        assert!(err.contains("messaging.inbox_queue_url"), "{err}");
    }

    #[test]
    fn secrets_are_not_shown_in_debug_output() {
        let shown = format!("{:?}", config(env_toml("local")));
        assert!(!shown.contains("platform-backend-secret"), "{shown}");
        assert!(!shown.contains("mysql://payroll:payroll"), "{shown}");
    }
}
