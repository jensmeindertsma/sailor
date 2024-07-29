use crate::configuration::Configuration;
use sail_config::{Configurable, CurrentConfiguration};
use sail_core::control::{Message, Reply, Request, Response};
use std::os::fd::FromRawFd;
use std::{env, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixListener,
    pin,
};
use tracing::{error, info};

pub struct Interface {
    socket: UnixListener,
    config: Arc<Configuration>,
}

impl Interface {
    pub fn attach_to_systemd_socket(config: Arc<Configuration>) -> Self {
        {
            use std::os::unix::net::UnixListener as StdUnixListener;

            let fd: i32 = match env::var("LISTEN_FDS") {
                Ok(fd) => fd.parse().expect("file descriptor should be valid integer"),
                Err(e) => {
                    error!("Missing LISTEN_FDS: {e}");
                    for var in env::vars() {
                        error!("var: {var:?}")
                    }

                    panic!("LISTEN_FDS invariant broken")
                }
            };

            info!("LISTEN_FDS={}", fd);

            if fd != 1 {
                panic!("Received unexpected file descriptor `{fd}` from systemd!")
            }

            // SAFETY: this comes from systemd
            let std_listener = unsafe { StdUnixListener::from_raw_fd(3) };
            std_listener
                .set_nonblocking(true)
                .expect("should be able to set non-blocking on the socket");

            info!("connected to socket");

            Self {
                socket: UnixListener::from_std(std_listener)
                    .expect("converting std::net::UnixListener to tokio::net::UnixListener"),
                config,
            }
        }
    }

    pub async fn handle_requests(&self) {
        while let Ok((mut stream, _)) = self.socket.accept().await {
            info!("new socket connection");
            let (reader, writer) = stream.split();
            pin!(writer);

            let mut lines = BufReader::new(reader).lines();
            while let Some(message) = lines
                .next_line()
                .await
                .expect("reading from the stream should succeed")
                .map(|l| {
                    serde_json::from_str::<Message>(&l)
                        .expect("deserialization of message should succeed")
                })
            {
                let cfg = self.config.get();

                let reply = Reply {
                    regarding: message.id,
                    response: match message.request {
                        Request::CreateApplication { application } => {
                            let mut applications = cfg.applications.clone();

                            if let Some(_existing_app) = applications
                                .iter()
                                .find(|a| a.hostname == application.hostname)
                            {
                                Response::Error {
                                    message: "application with this hostname already exists!"
                                        .into(),
                                }
                            } else {
                                applications.push(application.clone());

                                self.config
                                    .set(CurrentConfiguration {
                                        core: cfg.core.clone(),
                                        applications,
                                    })
                                    .await;

                                info!(
                                    "created application {} -> {}",
                                    application.hostname, application.address
                                );

                                Response::Success
                            }
                        }
                        Request::DeleteApplication { hostname } => {
                            let apps = cfg.applications.clone();

                            if !apps.iter().any(|a| a.hostname == hostname) {
                                Response::Error {
                                    message: format!("no app with hostname `{hostname}` exists"),
                                }
                            } else {
                                let new_applications = apps
                                    .into_iter()
                                    .filter(|a| a.hostname != hostname)
                                    .collect();

                                self.config
                                    .set(CurrentConfiguration {
                                        core: cfg.core.clone(),
                                        applications: new_applications,
                                    })
                                    .await;

                                info!("deleted application {hostname}");

                                Response::Success
                            }
                        }
                        Request::GetApplications => {
                            info!("request: getting applications..");
                            Response::Applications {
                                applications: cfg.applications.clone(),
                            }
                        }
                        Request::Status => {
                            info!("status request");
                            Response::Status {
                                port: self.config.get().core.port,
                                applications: self.config.get().applications.clone(),
                            }
                        }
                        Request::ValidateConfiguration => Response::Error {
                            message: "todo!".into(),
                        },
                    },
                };

                writer
                    .write_all(
                        format!(
                            "{}\n",
                            serde_json::to_string(&reply)
                                .expect("serialization of reply should succeed")
                        )
                        .as_bytes(),
                    )
                    .await
                    .expect("writing to the stream should succeed");
            }
        }
    }
}
