use std::fs;

#[derive(serde::Deserialize, Debug, Default)]
pub struct AppConfig {
    discord: DiscordSubConfig,
    database_path: String,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct DiscordSubConfig {
    token: Option<String>,
    guilds: Option<Vec<String>>,
}

impl AppConfig {
    pub fn from_file(path: &str, with_env: bool) -> Result<Self, crate::error::Error> {
        let bytes = fs::read(path)?;
        let mut config = serde_json::from_slice::<AppConfig>(&bytes)?;

        if config.discord.guilds.is_none() {
            config.discord.guilds = Some(vec![]);
        }

        if with_env {
            config = config.with_env();
        }

        if config.discord.guilds.as_ref().unwrap().is_empty() {
            return Err(crate::error::Error::bot("No guilds supplied."));
        }

        Ok(config)
    }

    pub fn with_env(mut self) -> Self {
        let guilds = self.discord.guilds.as_mut().unwrap();

        if self.discord.token.is_none() {
            let token = std::env::var("DISCORD_TOKEN");
            if let Ok(token) = token {
                self.discord.token = Some(token);
            }
        }

        if guilds.is_empty() {
            let guild = std::env::var("DISCORD_GUILD");
            if let Ok(guild) = guild {
                guilds.push(guild);
            }
        }

        self
    }

    // getters
    pub fn discord_token(&self) -> Option<&str> {
        self.discord.token.as_deref()
    }

    pub fn discord_guilds(&self) -> &[String] {
        self.discord.guilds.as_ref().unwrap()
    }
    pub fn database_path(&self) -> &str {
        &self.database_path
    }
}
