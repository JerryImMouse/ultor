use super::*;
use crate::services::{SS14AuthClientService, ServicesContainer};
use serenity::all::CommandOptionType;
use serenity::async_trait;
use serenity::builder::CreateCommandOption;
use crate::try_discord_unwrap;

#[derive(Debug)]
pub struct SummonCommand {
    ss14_client: std::sync::Arc<SS14AuthClientService>,
}

impl SummonCommand {
    pub fn new(services: &ServicesContainer) -> Self {
        Self {
            ss14_client: services.get_unsafe(),
        }
    }
}

#[async_trait]
impl DiscordCommandHandler for SummonCommand {
    fn definition(&self) -> DiscordCommandDefinition {
        DiscordCommandDefinition::new_global("summon", true, false)
    }

    fn registration(&self) -> CreateCommand {
        CreateCommand::new("summon")
            .name_localized("ru", "призвать")
            .description("Summons(pings) member by its in-game login")
            .description_localized("ru", "Пингует пользователя по игровому логину")
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "login", "In-game login")
                    .description_localized("ru", "Внутриигровой логин"),
            )
            .default_member_permissions(MANAGE_WEBHOOKS_SERVER_PERMISSION)
    }

    async fn handler(&self, opts: &[ResolvedOption]) -> DiscordCommandResponse {
        let login = try_discord_unwrap!(
            opts_get_login(opts),
            none => "Login is not specified",
            ephemeral => false
        );

        let user_id = try_discord_unwrap!(
            self.ss14_client.get_user_id(login).await,
            none => "Login is not specified",
            error => "Error occurred during UID fetch.",
            log => "Failed to get user ID.",
            ephemeral => false
        );

        let discord_id = try_discord_unwrap!(
            self.ss14_client.get_discord_id(user_id).await,
            none => "This account is not linked.",
            error => "Error occurred during DUID fetch.",
            log => "Failed to get user ID.",
            ephemeral => false
        );

        DiscordCommandResponse::followup_response(&format!("<@{}>", discord_id), false)
    }
}
