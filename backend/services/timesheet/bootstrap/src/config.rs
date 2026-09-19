use platform_service::config::{
    AuthConfig, DatabaseConfig, EmbeddedConfig, MessagingConfig, Required, SecretsConfig,
    ServerConfig, TelemetryConfig,
};
use serde::Deserialize;

/// 勤怠サービスの設定(config/timesheet/)。環境ごとの設定もバイナリに埋め込む
const EMBEDDED: EmbeddedConfig = EmbeddedConfig {
    env_prefix: "TIMESHEET",
    default: include_str!("../../../../../config/timesheet/default.toml"),
    envs: &[
        ("local", include_str!("../../../../../config/timesheet/local.toml")),
        ("dev", include_str!("../../../../../config/timesheet/dev.toml")),
        ("stg", include_str!("../../../../../config/timesheet/stg.toml")),
        ("prd", include_str!("../../../../../config/timesheet/prd.toml")),
    ],
};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub env: String,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub messaging: MessagingConfig,
    pub secrets: SecretsConfig,
    pub telemetry: TelemetryConfig,
}

/// `APP_ENV`(local/dev/stg/prd)を見て、埋め込みの default.toml に `{env}.toml` と
/// 環境変数 `TIMESHEET__SECTION__KEY` を重ねる
pub fn load_config() -> anyhow::Result<AppConfig> {
    platform_service::config::load(&EMBEDDED)
}

impl AppConfig {
    /// server を動かすのに要る値がそろっているかを確かめる。足りなければ起動時に止める
    pub fn validate_for_server(&self) -> anyhow::Result<()> {
        Required::default()
            .server(&self.server, &self.database, &self.secrets, &self.auth, &self.messaging)
            .value("messaging.inbox_queue_url", &self.messaging.inbox_queue_url)
            .check()
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
    fn values_given_by_the_infrastructure_are_required_in_aws() {
        let err = config(env_toml("dev")).validate_for_server().unwrap_err().to_string();
        assert!(err.contains("auth.issuer") && err.contains("messaging.inbox_queue_url"), "{err}");
    }
}
