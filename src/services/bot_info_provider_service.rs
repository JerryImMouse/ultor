use std::sync::RwLock;

#[derive(Debug)]
pub struct BotInfoProviderService {
    commands_processed: RwLock<u32>,
    last_processed_command_name: RwLock<&'static str>,
}

impl Default for BotInfoProviderService {
    fn default() -> Self {
        Self::new()
    }
}

impl BotInfoProviderService {
    pub fn new() -> Self {
        Self {
            commands_processed: RwLock::new(0),
            last_processed_command_name: RwLock::new("none"),
        }
    }

    pub fn commands_processed(&self) -> u32 {
        *self.commands_processed.read().unwrap()
    }

    pub fn last_processed_command_name(&self) -> &'static str {
        *self.last_processed_command_name.read().unwrap()
    }

    pub fn inc_commands_processed(&self) {
        let mut lock = self.commands_processed.write().unwrap();
        *lock += 1;
    }

    pub fn set_last_processed_command_name(&self, command_name: &'static str) {
        *self.last_processed_command_name.write().unwrap() = command_name;
    }
}
