// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ Crate: common                                                       ┃
// ┃ Purpose: Shared logging and configuration utilities                 ┃
// ┃ Author: EricA1019                                                   ┃
// ┃ Last Updated: 2025-09-02                                           ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

use anyhow::Result;
use serde::Deserialize;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_auto_start_watchers")]
    pub auto_start_watchers: bool,
}

fn default_auto_start_watchers() -> bool { true }

/// Initialize tracing subscriber with env filter.
pub fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env_lossy());
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_target(true).with_line_number(true))
        .init();
}

/// Load configuration from config/{default,local}.yaml with env overrides.
pub fn load_config() -> Result<AppConfig> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("config/default.yaml").required(false))
        .add_source(config::File::with_name("config/local.yaml").required(false))
        .add_source(config::Environment::with_prefix("APP").separator("__"))
        .build()?;
    let cfg: AppConfig = settings.try_deserialize()?;
    Ok(cfg)
}

//EOF