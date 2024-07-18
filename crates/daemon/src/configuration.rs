use sailor_config::{Configurable, CoreConfiguration, CurrentConfiguration};
use sailor_core::application::Application;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tracing::{error, info};

const DEFAULT_PORT: u16 = 4250;

pub struct Configuration {
    options: Mutex<Arc<CurrentConfiguration>>,
}

impl Configurable for Configuration {
    fn get(&self) -> Arc<CurrentConfiguration> {
        Arc::clone(
            &*self
                .options
                .lock()
                .expect("should be able to get lock on configuration for retrieval"),
        )
    }

    async fn set(&self, new: CurrentConfiguration) {
        *self
            .options
            .lock()
            .expect("should be able to get lock on configuration for modification") = Arc::new(new);

        self.save().await;
    }
}

impl Configuration {
    pub async fn from_filesystem() -> Self {
        if let Ok(m) = fs::metadata("/etc/sailor").await {
            if !m.is_dir() {
                panic!("Unexpected file at `/etc/sailor`, please remove")
            }
        }

        let core_configuration: CoreConfiguration =
            match fs::read_to_string("/etc/sailorconfiguration.toml")
                .await
                .map(|s| toml::from_str(&s).expect("Configuration file should be valid TOML"))
            {
                Ok(config) => config,
                Err(_) => CoreConfiguration { port: DEFAULT_PORT },
            };

        let applications = match fs::read_dir("/etc/sailor/applications").await {
            Ok(mut entries) => {
                let mut apps = Vec::new();

                while let Some(entry) = entries
                    .next_entry()
                    .await
                    .expect("should be able to read application configuration directory entries")
                {
                    let file_name = entry
                        .file_name()
                        .into_string()
                        .expect("parsing file name should succeed");

                    if !file_name.ends_with(".toml") {
                        panic!("unexpected file extension: {file_name}")
                    }

                    let str =
                        fs::read_to_string(format!("/etc/sailorapplications/{file_name}")).await;
                    let str = match str {
                        Ok(s) => s,
                        Err(e) => {
                            error!("failed to read `{file_name}`: {e}");
                            continue;
                        }
                    };

                    let config: Application = match toml::from_str(&str) {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Failed to parse config file `{file_name}`: {e}");
                            continue;
                        }
                    };

                    apps.push(config);
                }

                apps
            }
            Err(_) => Vec::new(),
        };

        let cfg = Self {
            options: Mutex::new(Arc::new(CurrentConfiguration {
                core: core_configuration,
                applications,
            })),
        };

        cfg.save().await;

        cfg
    }

    pub async fn save(&self) {
        info!("Saving config:  {:?}", self.get());

        match fs::metadata("/etc/sail").await {
            Ok(m) => {
                if !m.is_dir() {
                    fs::remove_file("/etc/sail").await.unwrap();
                    fs::create_dir("/etc/sail").await.unwrap();
                };

                // The directory already exists.
            }
            Err(_) => {
                // The directory does not yet exist, create it.!
                fs::create_dir("/etc/sail").await.unwrap();
            } //
        }

        let cfg = self.get();
        let core =
            toml::to_string_pretty(&cfg.core).expect("internal config should be serializable");

        fs::write("/etc/sailorconfiguration.toml", core)
            .await
            .unwrap();

        match fs::metadata("/etc/sailorapplications").await {
            Ok(m) => {
                if !m.is_dir() {
                    fs::remove_file("/etc/sailorapplications").await.unwrap();
                    fs::create_dir("/etc/sailorapplications").await.unwrap();
                }
            }
            Err(_) => {
                // does not exist, let's make the directory
                fs::create_dir("/etc/sailorapplications").await.unwrap();
            }
        }

        for app in cfg.applications.iter() {
            fs::write(
                format!("/etc/sailorapplications/{}.toml", app.hostname),
                toml::to_string_pretty(app).unwrap(),
            )
            .await
            .unwrap();
        }
    }
}
