use super::*;
use crate::services::auth_client_service::SS14AuthClientService;
use crate::services::ServicesContainer;
use crate::utils::{gen_random_uuid, RED_COLOR};
use log::error;
use serenity::all::CommandOptionType;
use serenity::async_trait;
use serenity::builder::CreateCommandOption;

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
        let login = match opts_get_login(opts) {
            Some(l) => l,
            None => {
                return DiscordCommandResponse::followup_embed_response(
                    "Login is not specified",
                    None,
                    Some(RED_COLOR),
                    true,
                )
            }
        };

        let user_id_result = self.ss14_client.get_user_id(login).await;
        let user_id = match user_id_result {
            Ok(Some(user_id)) => user_id,
            Ok(None) => {
                return DiscordCommandResponse::followup_embed_response(
                    "Login is not specified",
                    None,
                    Some(RED_COLOR),
                    true,
                )
            }
            Err(e) => {
                let uuid = gen_random_uuid();
                error!("{}. Failed to get user ID: {}", uuid, e);
                return DiscordCommandResponse::followup_embed_response(
                    &format!("Error occurred during UID fetch.\nError ID: `{}`", uuid),
                    None,
                    Some(RED_COLOR),
                    true,
                );
            }
        };

        let discord_id_result = self.ss14_client.get_discord_id(user_id).await;
        let discord_id = match discord_id_result {
            Ok(Some(discord_id)) => discord_id,
            Ok(None) => {
                return DiscordCommandResponse::followup_embed_response(
                    "This account is not linked.",
                    None,
                    Some(RED_COLOR),
                    true,
                )
            }
            Err(e) => {
                let uuid = gen_random_uuid();
                error!("{}. Failed to get discord ID: {}", uuid, e);
                return DiscordCommandResponse::followup_embed_response(
                    &format!("Error occurred during DUID fetch.\nError ID: `{}`", uuid),
                    None,
                    Some(RED_COLOR),
                    true,
                );
            }
        };

        DiscordCommandResponse::followup_response(&format!("<@{}>", discord_id), false)
    }
}
