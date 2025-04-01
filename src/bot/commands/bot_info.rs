use crate::bot::commands::{
    DiscordCommandDefinition, DiscordCommandHandler, DiscordCommandResponse,
};
use crate::services::bot_info_provider_service::BotInfoProviderService;
use crate::services::ServicesContainer;
use serenity::all::{Color, CreateCommand, ResolvedOption};
use serenity::async_trait;

#[derive(Debug)]
pub struct BotInfo {
    bot_info_provider_service: std::sync::Arc<BotInfoProviderService>,
}

impl BotInfo {
    pub fn new(services: &ServicesContainer) -> Self {
        Self {
            bot_info_provider_service: services.get_unsafe(),
        }
    }
}

#[async_trait]
impl DiscordCommandHandler for BotInfo {
    fn definition(&self) -> DiscordCommandDefinition {
        DiscordCommandDefinition::new_global("bot_info", false)
    }

    fn registration(&self) -> CreateCommand {
        CreateCommand::new("bot_info")
            .description("Get information about the bot.")
            .description_localized("ru", "Получить информацию о боте.")
    }

    async fn handler(&self, _opts: &[ResolvedOption]) -> DiscordCommandResponse {
        let commands_processed = self.bot_info_provider_service.commands_processed();
        let last_processed_command_name =
            self.bot_info_provider_service.last_processed_command_name();

        DiscordCommandResponse::default_embed_response(
            &format!(
                "Commands Processed: {}\nLast processed command name: {}",
                commands_processed, last_processed_command_name
            ),
            None,
            Some(Color::from_rgb(111, 50, 168)),
            true,
        )
    }
}
