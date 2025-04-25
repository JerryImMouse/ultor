static APP_VERSION: &str = env!("CARGO_PKG_VERSION");
static DEFAULT_LOG_LEVEL: &str = "debug";

static DEFAULT_CONFIG_PATH: &str = "config.json";
static DEFAULT_DEV_CONFIG_PATH: &str = "config.dev.json";
const OVERRIDE_CONFIG_PATH: Option<&str> = option_env!("CONFIG_PATH");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = env_logger::Env::new()
        .filter_or("RUST_LOG", format!("ultor={}", DEFAULT_LOG_LEVEL))
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let cfg_path = {
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

        match ultor::ConfigBuilder::new(primary_path.to_string()).init() {
            Ok(_) => primary_path.to_string(),
            Err(_) => {
                ultor::ConfigBuilder::new(DEFAULT_CONFIG_PATH.to_string()).init()?;
                primary_path.to_string()
            },
        }
    };

    log::debug!("Successfully loaded config: {:?}", cfg_path);

    // register basic IoC
    let container = ultor::services::ServicesContainer::new();
    ultor::initialize_services(&container).await?;

    log_runtime(&cfg_path);

    let bot = ultor::DiscordApp::new(ultor::command_definitions(&container), &container)?;
    bot.start().await?;

    Ok(())
}

fn log_runtime(cfg_path: &str) {
    log::info!("Version: {}", APP_VERSION);
    log::info!("Configuration: {}", cfg_path);
}
