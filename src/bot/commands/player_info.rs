use std::collections::HashMap;
use super::*;
use crate::services::auth_client_service::SS14AuthClientService;
use crate::services::ServicesContainer;
use log::error;
use serde_json::Value;
use serenity::all::{Color, CommandOptionType, CreateCommand, CreateCommandOption, PartialMember, ResolvedOption, User, UserId};
use serenity::async_trait;
use sqlx::types::JsonValue;
use crate::config::ConfigValue;
use crate::services::ss14_database_service::SS14DatabaseService;
use crate::utils::{gen_random_color, gen_random_uuid, RED_COLOR};

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

#[async_trait]
impl DiscordCommandHandler for PlayerInfoCommand {
    fn definition(&self) -> DiscordCommandDefinition {
        DiscordCommandDefinition::new_global("player_info", true, true)
    }

    fn registration(&self) -> CreateCommand {
        CreateCommand::new("player_info")
            .name_localized("ru", "игрок")
            .description("Fetches player info from all available sources.")
            .description_localized("ru", "Получает информацию об игрок из всех возможных источников.")
            .add_option(CreateCommandOption::new(
                CommandOptionType::User,
                "user",
                "User to fetch info about.",
            ))
            .default_member_permissions(MANAGE_WEBHOOKS_SERVER_PERMISSION)
    }

    async fn handler(&self, opts: &[ResolvedOption]) -> DiscordCommandResponse {
        let not_found = |reason: &str| {
            DiscordCommandResponse::followup_embed_response(
                reason,
                None,
                Some(RED_COLOR),
                true,
            )
        };

        let user_id = opts.iter().find_map(|opt| match (&opt.name[..], &opt.value) {
            ("user", ResolvedValue::User(user, _)) => Some(user.id),
            _ => None,
        });

        let user_id = match user_id {
            Some(user_id) => user_id,
            None => return not_found("🚫 Could not resolve the specified user."),
        };

        let ss14_user_id = match self
            .ss14_client
            .get_user_id_from_discord(user_id.to_string())
            .await
        {
            Ok(Some(user_id)) => user_id,
            Ok(None) => return not_found("🔍 No linked SS14 account found for this user."),
            Err(e) => {
                let err_id = gen_random_uuid();
                error!("{}. Failed to get UID by Discord ID. Error: {}", err_id, e);
                return DiscordCommandResponse::followup_embed_response(
                    "❌ An error occurred while fetching UUID.",
                    Some(&err_id.to_string()),
                    Some(RED_COLOR),
                    true,
                );
            }
        };

        let in_game_login = match self.ss14_db.get_login(ss14_user_id).await {
            Ok(Some(login)) => login,
            Ok(None) => return not_found("❔ User has no known in-game login."),
            Err(e) => {
                let err_id = gen_random_uuid();
                error!("{}. Failed to get login by Discord ID. Error: {}", err_id, e);
                return DiscordCommandResponse::followup_embed_response(
                    "❌ An error occurred while fetching login.",
                    Some(&err_id.to_string()),
                    Some(RED_COLOR),
                    true,
                );
            }
        };
        let extra_data = format_extra_data(&user_id.to_string(), &self.ss14_client).await;
        let extra_data = match extra_data {
            Ok(extra_data) => extra_data,
            Err(e) => {
                let err_id = gen_random_uuid();
                error!("{}. Failed to get extra data by Discord ID. Error: {}", err_id, e);
                return DiscordCommandResponse::followup_embed_response(
                    "❌ An error occurred while fetching extra data.",
                    Some(&err_id.to_string()),
                    Some(RED_COLOR),
                    true,
                );
            }
        };

        DiscordCommandResponse::followup_embed_response(
            &format!("🧑‍🚀 **In-Game Login:** `{}`\n🧾 **Extra Data:** \n{}", in_game_login, extra_data),
            None,
            Some(gen_random_color()),
            true,
        )
    }
}

async fn format_extra_data(
    discord_id: &str,
    ss14_client: &std::sync::Arc<SS14AuthClientService>
) -> Result<String, crate::error::Error> {
    let capitalize_key = |key: &str| {
        key.split('_')
            .map(|part| {
                let mut c = part.chars();
                match c.next() {
                    Some(first) => first.to_uppercase().chain(c).collect(),
                    None => String::new(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    };

    let value = ss14_client.get_extra_data(discord_id.to_string()).await?;

    match value {
        Some(value) => {
            let obj = value.as_object().unwrap();
            let mut result = String::new();

            for (k, v) in obj {
                match v {
                    Value::String(s) => {
                        result.push_str(&format!("{}: {}\n", capitalize_key(k), s));
                    }
                    Value::Number(n) if n.is_i64() || n.is_u64() => {
                        result.push_str(&format!("{}: {}\n", capitalize_key(k), n));
                    }
                    _ => {}
                }
            }

            if result.is_empty() {
                Ok("No extra data found.".to_string())
            } else {
                Ok(result)
            }
        }
        None => Ok("No extra data found.".to_string()),
    }
}
