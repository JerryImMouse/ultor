pub mod commands;

use crate::bot::commands::{DiscordCommandHandler, DiscordCommandResponse};
use crate::services::ServicesContainer;
use crate::{config::AppConfig, error::Error};
use log::{debug, error, info};
use serenity::all::{
    Command, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId,
    Interaction, Ready,
};
use serenity::async_trait;
use serenity::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct DiscordApp {
    configuration: AppConfig,
    registrations: Vec<CreateCommand>,
    global_registrations: Vec<CreateCommand>,
    guilds: Vec<GuildId>,

    handlers_map: BTreeMap<String, Arc<dyn DiscordCommandHandler + Send + Sync>>,
    handlers: Vec<Arc<dyn DiscordCommandHandler + Send + Sync>>,
}

#[async_trait]
impl EventHandler for DiscordApp {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let bot_name = ready.user.name.as_str();

        info!("{} connected to discord API. Working...", bot_name);
        info!(
            "Registering {} commands...",
            self.registrations.len() + self.global_registrations.len()
        );
        debug!("Guilds: {:?}", ready.guilds);

        for guild in &self.guilds {
            let commands = guild
                .set_commands(&ctx.http, self.registrations.clone())
                .await;
            if let Err(why) = commands {
                error!("Error pushing commands to {}: {}", guild, why);
                continue;
            }

            debug!(
                "Registered {} commands to {}",
                commands.unwrap().len(),
                guild
            );
        }

        let commands =
            Command::set_global_commands(&ctx.http, self.global_registrations.clone()).await;
        if let Err(why) = &commands {
            error!("Error pushing global commands: {}", why);
        }

        debug!("Registered {} global commands", commands.unwrap().len());
        info!("Finished commands registering. Listening for incoming interactions...");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(cmd) = interaction {
            debug!(
                "Received `command` interaction from {}, command: {}. Processing...",
                cmd.user.name, cmd.data.name
            );

            let handler = self.handlers_map.get(&cmd.data.name);
            if handler.is_none() {
                let result = cmd
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content("Command not found!"),
                        ),
                    )
                    .await;

                if let Err(e) = result {
                    error!("Error responding to discord: {e}");
                }

                return;
            }

            let handler = handler.unwrap();

            // 2 branches
            // deferred and default
            if handler.definition().is_deferred {
                if let Err(e) = cmd
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Defer(
                            CreateInteractionResponseMessage::new()
                                .content("Your request is processing...")
                                .ephemeral(handler.definition().is_ephemeral),
                        ),
                    )
                    .await
                {
                    error!("Error creating defer response: {e}");
                    return;
                }

                let opts = &cmd.data.options();
                let response = handler.handler(opts).await;
                match response {
                    DiscordCommandResponse::Default(_) => {
                        error!("Deferred command returned default response!");
                    }
                    DiscordCommandResponse::Followup(response) => {
                        if let Err(e) = cmd.create_followup(&ctx.http, response).await {
                            error!("Error sending followup command: {e}");
                        }
                    }
                }
                return;
            }

            let opts = &cmd.data.options();
            let response = handler.handler(opts).await;
            match response {
                DiscordCommandResponse::Default(response) => {
                    if let Err(e) = cmd.create_response(&ctx.http, response).await {
                        error!("Error sending followup command: {e}");
                    }
                }
                DiscordCommandResponse::Followup(_) => {
                    error!("Default command returned deferred response!");
                }
            }
        }
    }
}

impl DiscordApp {
    pub fn new(
        config: AppConfig,
        command_defs: Vec<Arc<dyn DiscordCommandHandler + Send + Sync>>,
        _services: &ServicesContainer,
    ) -> Result<Self, crate::error::Error> {
        let mut app = Self {
            registrations: vec![],
            global_registrations: vec![],
            guilds: Vec::with_capacity(config.discord_guilds().len()),
            configuration: config,
            handlers: vec![],
            handlers_map: BTreeMap::new(),
        };

        app.construct_commands(command_defs)?;
        app.populate_registrations()?;
        app.populate_guilds()?;

        Ok(app)
    }

    pub async fn start(self) -> Result<(), Error> {
        let token = self
            .configuration
            .discord_token()
            .ok_or_else(|| Error::bot("Discord token is empty"))?;

        let mut client = Client::builder(token, GatewayIntents::empty())
            .event_handler(self)
            .await?;

        client.start().await?;
        Ok(())
    }

    fn construct_commands(
        &mut self,
        commands: Vec<Arc<dyn DiscordCommandHandler + Send + Sync>>,
    ) -> Result<(), Error> {
        for command in commands {
            self.handlers_map
                .insert(command.definition().name.to_string(), Arc::clone(&command));
            self.handlers.push(command);
        }

        debug!("Constructed commands: {:?}", self.handlers);
        debug!("Constructed commands tree: {:?}", self.handlers_map);

        Ok(())
    }

    fn populate_registrations(&mut self) -> Result<(), Error> {
        for command in &self.handlers {
            if command.definition().is_global {
                self.global_registrations.push(command.registration());
            } else {
                self.registrations.push(command.registration());
            }
        }

        Ok(())
    }

    fn populate_guilds(&mut self) -> Result<(), Error> {
        for guild_id in self.configuration.discord_guilds() {
            let guild_id = guild_id.parse::<u64>()?;
            let guild_id = GuildId::from(guild_id);
            self.guilds.push(guild_id);
        }

        Ok(())
    }
}
