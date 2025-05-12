use super::*;
use crate::services::{SS14AuthClientService, SS14DatabaseService, ServicesContainer};
use crate::try_discord_unwrap;
use crate::utils::{format_extra_data, gen_random_color};
use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption};

#[derive(Debug)]
pub struct PlayerInfoCommand {
    ss14_client: std::sync::Arc<SS14AuthClientService>,
    ss14_db: std::sync::Arc<SS14DatabaseService>,
}

impl PlayerInfoCommand {
    pub fn new(services: &ServicesContainer) -> Self {
        Self {
            ss14_client: services.get_unsafe(),
            ss14_db: services.get_unsafe(),
        }
    }
}

#[serenity::async_trait]
impl DiscordCommandHandler for PlayerInfoCommand {
    fn definition(&self) -> DiscordCommandDefinition {
        DiscordCommandDefinition::new_global("player_info", true, true)
    }

    fn registration(&self) -> CreateCommand {
        CreateCommand::new("player_info")
            .name_localized("ru", "игрок")
            .description("Fetches player info from all available sources.")
            .description_localized(
                "ru",
                "Получает информацию об игрок из всех возможных источников.",
            )
            .add_option(CreateCommandOption::new(
                CommandOptionType::User,
                "user",
                "User to fetch info about.",
            ))
            .default_member_permissions(MANAGE_WEBHOOKS_SERVER_PERMISSION)
    }

    async fn handler(&self, opts: &[ResolvedOption]) -> DiscordCommandResponse {
        let user_id = opts.iter().find_map(|opt| match (opt.name, &opt.value) {
            ("user", ResolvedValue::User(user, _)) => Some(user.id),
            _ => None,
        });

        let user_id = try_discord_unwrap!(user_id,
            none => "🚫 Could not resolve the specified user.",);

        let ss14_user_id = try_discord_unwrap!(
            self.ss14_client.get_user_id_from_discord(user_id.to_string()).await,
            none => "🔍 No linked SS14 account found for this user.",
            error => "❌ An error occurred while fetching UUID.",
            log => "Failed to get UID by Discord ID.",
            ephemeral => true
        );

        let in_game_login = try_discord_unwrap!(
            self.ss14_db.get_login(ss14_user_id).await,
            none => "❔ User has no known in-game login.",
            error => "❌ An error occurred while fetching login.",
            log => "Failed to get login by Discord ID.",
            ephemeral => true
        );

        let extra_data = format_extra_data(&user_id.to_string(), &self.ss14_client).await;
        let extra_data = try_discord_unwrap!(
            extra_data,
            error => "❌ An error occurred while fetching extra data.",
            log => "Failed to get extra data by Discord ID.",
            ephemeral => true
        );

        DiscordCommandResponse::followup_embed_response(
            &format!(
                "🧑‍🚀 **In-Game Login:** `{}`\n🧾 **Extra Data:** \n{}",
                in_game_login, extra_data
            ),
            None,
            Some(gen_random_color()),
            true,
        )
    }
}
