use super::*;
use crate::services::{SS14AuthClientService, ServicesContainer};
use crate::{extract_discord_arg, try_discord_unwrap};
use log::error;
use serenity::all::{Color, CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption};
use serenity::async_trait;

#[derive(Debug)]
pub struct UserIdCommand {
    ss14_client: std::sync::Arc<SS14AuthClientService>,
}

impl UserIdCommand {
    pub fn new(services: &ServicesContainer) -> Self {
        Self {
            ss14_client: services.get_unsafe(),
        }
    }
}

#[async_trait]
impl DiscordCommandHandler for UserIdCommand {
    fn definition(&self) -> DiscordCommandDefinition {
        DiscordCommandDefinition::new_global("user_id", true, true)
    }

    fn registration(&self) -> CreateCommand {
        CreateCommand::new("user_id")
            .description("Fetches SS14 user ID by in-game login")
            .description_localized("ru", "Получает SS14 ID пользователя по игровому логину.")
            .add_option(CreateCommandOption::new(
                CommandOptionType::String,
                "login",
                "In-game login",
            ))
            .default_member_permissions(MANAGE_WEBHOOKS_SERVER_PERMISSION)
    }

    async fn handler(&self, opts: &[ResolvedOption]) -> DiscordCommandResponse {
        let mut login = try_discord_unwrap!(
            extract_discord_arg!(opts, "login", String),
            none => "Login is not specified",
            ephemeral => false
        );

        let found = self.ss14_client.get_user_id(login).await;
        match found {
            Ok(None) => login = "Such player doesn't exist.".to_string(),
            Ok(Some(user_id)) => login = format!("**User ID:** {}", user_id),
            Err(e) => {
                error!("Failed to fetch SS14 user ID: {}", e);
                login = "Error occurred during fetching...".to_string()
            }
        }

        DiscordCommandResponse::followup_embed_response(
            login.as_ref(),
            None,
            Some(Color::from_rgb(255, 255, 0)),
            true,
        )
    }
}
