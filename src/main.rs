use env_logger::Env;
use log::{debug, info};
use std::sync::Arc;
use ultor::bot::commands::ping::PingCommand;
use ultor::bot::commands::DiscordCommandHandler;
use ultor::bot::DiscordApp;
use ultor::config::AppConfig;
use ultor::error::Error;
use ultor::services::bot_db_service::BotDatabaseService;
use ultor::services::ServicesContainer;

static APP_VERSION: &str = env!("CARGO_PKG_VERSION");
static DEFAULT_LOG_LEVEL: &str = "debug";

static DEFAULT_CONFIG_PATH: &str = "config.json";
static DEFAULT_DEV_CONFIG_PATH: &str = "config.dev.json";
const OVERRIDE_CONFIG_PATH: Option<&str> = option_env!("CONFIG_PATH");

fn command_definitions(
    services: &ServicesContainer,
) -> Vec<Arc<dyn DiscordCommandHandler + Send + Sync>> {
    vec![
        Arc::new(PingCommand)
    ]
}

async fn initialize_services(config: &AppConfig, services_container: &ServicesContainer) -> Result<(), Error> {
    let db_service = BotDatabaseService::new(
        config.database_path().to_string(),
        "./migrations".to_string()
    ).await?;

    services_container.register(db_service);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::new()
        .filter_or("RUST_LOG", format!("ultor={}", DEFAULT_LOG_LEVEL))
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let (config, cfg_path): (AppConfig, String) = {
        let primary_path = if cfg!(debug_assertions)
            && std::fs::exists(DEFAULT_DEV_CONFIG_PATH).unwrap()
            && OVERRIDE_CONFIG_PATH.is_none()
        {
            DEFAULT_DEV_CONFIG_PATH
        } else if OVERRIDE_CONFIG_PATH.is_some()
            && std::fs::exists(OVERRIDE_CONFIG_PATH.unwrap()).unwrap()
        {
            OVERRIDE_CONFIG_PATH.unwrap()
        } else {
            DEFAULT_CONFIG_PATH
        };

        match AppConfig::from_file(primary_path, true) {
            Ok(config) => Ok((config, primary_path.to_string())),
            Err(_) => AppConfig::from_file(DEFAULT_CONFIG_PATH, true)
                .map(|v| (v, primary_path.to_string())),
        }
    }?;

    debug!("Successfully loaded config: {:?}", config);

    // register basic IoC
    let container = ServicesContainer::new();
    initialize_services(&config, &container).await?;

    log_runtime(&cfg_path);

    let bot = DiscordApp::new(config, command_definitions(&container), &container)?;
    bot.start().await?;

    Ok(())
}

fn log_runtime(cfg_path: &str) {
    info!("Version: {}", APP_VERSION);
    info!("Configuration: {}", cfg_path);
}
