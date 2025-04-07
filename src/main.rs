use env_logger::Env;
use log::{debug, info};
use std::sync::Arc;
use ultor::bot::commands::ping::PingCommand;
use ultor::bot::commands::player_info::PlayerInfoCommand;
use ultor::bot::commands::summon::SummonCommand;
use ultor::bot::commands::unlink::UnLinkCommand;
use ultor::bot::commands::user_id::UserIdCommand;
use ultor::bot::commands::DiscordCommandHandler;
use ultor::bot::DiscordApp;
use ultor::config::{Config, ConfigBuilder};
use ultor::config_get;
use ultor::error::Error;
use ultor::services::auth_client_service::SS14AuthClientService;
use ultor::services::bot_db_service::BotDatabaseService;
use ultor::services::ss14_database_service::SS14DatabaseService;
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
        Arc::new(PingCommand),
        Arc::new(UserIdCommand::new(services)),
        Arc::new(SummonCommand::new(services)),
        Arc::new(PlayerInfoCommand::new(services)),
        Arc::new(UnLinkCommand::new(services)),
    ]
}

async fn initialize_services(
    config: &Config,
    services_container: &ServicesContainer,
) -> Result<(), Error> {
    let bot_db_path = config_get!(config, "database.bot_database_path", as_str).unwrap();

    let db_service =
        BotDatabaseService::new(bot_db_path.to_string(), "./migrations".to_string()).await?;
    services_container.register(db_service);

    let ss14_db_uri = config_get!(config, "database.ss14_database_url", as_str).unwrap();
    services_container.register(SS14DatabaseService::new(ss14_db_uri.to_string())?);

    let discord_auth_uri = config_get!(config, "auth.discord_auth_uri", as_str).unwrap();
    let discord_auth_token = config_get!(config, "auth.discord_auth_token", as_str).unwrap();
    let ss14_auth_uri = config_get!(config, "auth.ss14_auth_uri", as_str).unwrap();
    services_container.register(SS14AuthClientService::new(
        discord_auth_uri.to_string(),
        discord_auth_token.to_string(),
        ss14_auth_uri.to_string(),
    )?);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::new()
        .filter_or("RUST_LOG", format!("ultor={}", DEFAULT_LOG_LEVEL))
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let (config, cfg_path): (Config, String) = {
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

        match ConfigBuilder::new(primary_path.to_string()).build() {
            Ok(config) => Ok((config, primary_path.to_string())),
            Err(_) => ConfigBuilder::new(DEFAULT_CONFIG_PATH.to_string())
                .build()
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
