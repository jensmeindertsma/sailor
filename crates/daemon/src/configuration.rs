use std::sync::{Arc, Mutex};

pub struct Configuration {
    settings: Mutex<Settings>,
}

impl Configuration {
    pub fn from_filesystem() -> Result<Self, ConfigurationError> {
        Ok(Self {
            settings: Mutex::new(Settings {}),
        })
    }

    pub fn get(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn set(&self, new_settings: Settings) {
        *self.settings.lock().unwrap() = new_settings;
    }
}

#[derive(Clone, Debug)]
pub struct Settings {}

#[derive(Debug)]
pub enum ConfigurationError {}
