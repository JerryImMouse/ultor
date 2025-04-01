use super::*;
use serenity::async_trait;

#[derive(Debug)]
pub struct PingCommand;

#[async_trait]
impl DiscordCommandHandler for PingCommand {
    fn definition(&self) -> DiscordCommandDefinition {
        DiscordCommandDefinition::new_global("ping", false)
    }

    fn registration(&self) -> CreateCommand {
        CreateCommand::new("ping")
            .description("Pings the bot.")
            .description_localized("ru", "Пингует бота")
    }

    async fn handler(&self, _opts: &[ResolvedOption]) -> DiscordCommandResponse {
        DiscordCommandResponse::default_response("Pong!", false)
    }
}
