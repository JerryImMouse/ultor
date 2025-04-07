use super::*;
use crate::services::auth_client_service::SS14AuthClientService;
use crate::services::ServicesContainer;
use crate::utils::gen_random_uuid;
use log::error;
use serenity::all::CommandOptionType;
use serenity::async_trait;
use serenity::builder::CreateCommandOption;

#[derive(Debug)]
pub struct UnLinkCommand {
    ss14_client: std::sync::Arc<SS14AuthClientService>,
}

impl UnLinkCommand {
    pub fn new(services: &ServicesContainer) -> Self {
        Self {
            ss14_client: services.get_unsafe(),
        }
    }
}

#[async_trait]
impl DiscordCommandHandler for UnLinkCommand {
    fn definition(&self) -> DiscordCommandDefinition {
        DiscordCommandDefinition::new_global("unlink", true, true)
    }

    fn registration(&self) -> CreateCommand {
        CreateCommand::new("unlink")
            .name_localized("ru", "отвязать")
            .description("Unlinks a player's account by Discord ID or in-game login")
            .description_localized(
                "ru",
                "Отвязывает аккаунт игрока по Discord ID или игровому логину",
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "type", "The type of id field")
                    .name_localized("ru", "тип")
                    .description_localized("ru", "Тип ID поля")
                    .add_string_choice("Discord", "discord")
                    .add_string_choice("SS14", "ss14"),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "id",
                    "ID field, its type depends on `type` field",
                )
                .description_localized("ru", "ID поле, тип зависит от поля `тип`."),
            )
            .default_member_permissions(MANAGE_WEBHOOKS_SERVER_PERMISSION)
    }

    async fn handler(&self, opts: &[ResolvedOption]) -> DiscordCommandResponse {
        let mut id_type: Option<String> = None;
        let mut id: Option<String> = None;

        for opt in opts {
            match (opt.name, &opt.value) {
                ("type", ResolvedValue::String(t)) => id_type = Some(t.to_string()),
                ("id", ResolvedValue::String(t)) => id = Some(t.to_string()),
                _ => {}
            }
        }

        if id_type.is_none() || id.is_none() {
            return DiscordCommandResponse::followup_response(
                "No ID or ID type fields supplied",
                true,
            );
        }

        let id_type = id_type.unwrap();
        let id = id.unwrap();

        let method = match id_type.as_str() {
            "discord" => "discord",
            "ss14" => "uid",
            _ => return DiscordCommandResponse::followup_response("Invalid ID type", true),
        };

        let result = self.ss14_client.delete_record(method.to_string(), id).await;
        match result {
            Ok(Some(_)) => {
                DiscordCommandResponse::followup_response("Successfully unlinked account.", true)
            }
            Ok(None) => {
                DiscordCommandResponse::followup_response("No such linked account exist.", true)
            }
            Err(e) => {
                let err_id = gen_random_uuid();
                error!("{}. Failed to delete record. Error: {}", err_id, e);
                DiscordCommandResponse::followup_response(
                    &format!(
                        "Error occurred while trying to delete record.\nError ID: {}",
                        err_id
                    ),
                    true,
                )
            }
        }
    }
}
