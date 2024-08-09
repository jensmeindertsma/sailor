use std::{collections::HashMap, sync::Mutex};

pub struct Configuration {
    settings: Mutex<Settings>,
}

impl Configuration {
    pub fn from_filesystem() -> Result<Self, ConfigurationError> {
        let mut applications = HashMap::new();

        applications.insert(
            "helloworld".to_owned(),
            Application {
                name: String::from("HelloWorldApp"),
            },
        );

        Ok(Self {
            settings: Mutex::new(Settings { applications }),
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
pub struct Settings {
    pub applications: HashMap<String, Application>,
}

#[derive(Clone, Debug)]
struct Application {
    pub name: String,
}

#[derive(Debug)]
pub enum ConfigurationError {}
