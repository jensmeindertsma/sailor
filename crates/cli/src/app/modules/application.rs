use crate::app::{controller::Controller, Failure};
use sailor_core::{
    application::Application,
    control::{Request, Response},
};

pub fn application(
    controller: &mut Controller,
    mut arguments: impl Iterator<Item = String>,
) -> Result<(), Failure> {
    let subcommand = arguments.next().ok_or(Failure::MissingCommand)?;

    match subcommand.as_str() {
        "create" => {
            let hostname = arguments.next().ok_or(Failure::MissingCommand)?;
            let address = arguments.next().ok_or(Failure::MissingCommand)?;

            let request = Request::CreateApplication {
                application: Application {
                    hostname,
                    address: address.parse().expect("address should be valid"),
                },
            };

            let response = controller.request(request);

            match response {
                Response::Error { message } => {
                    eprintln!("ERROR:  {message}")
                }
                Response::Success => {
                    println!("SUCCESS!")
                }
                other => panic!("Unexpected response: {other:?}"),
            }
        }
        "delete" => {
            let hostname = arguments.next().ok_or(Failure::MissingCommand)?;

            let request = Request::DeleteApplication { hostname };

            let response = controller.request(request);

            match response {
                Response::Error { message } => {
                    eprintln!("ERROR:  {message}")
                }
                Response::Success => {
                    println!("SUCCESS!")
                }
                other => panic!("Unexpected response: {other:?}"),
            }
        }

        "list" => {
            let request = Request::GetApplications;

            let response = controller.request(request);

            match response {
                Response::Error { message } => {
                    eprintln!("ERROR:  {message}")
                }
                Response::Applications { applications } => {
                    println!("SUCCESS!");
                    println!("{:#?}", applications)
                }
                other => panic!("Unexpected response: {other:?}"),
            }
        }
        _ => return Err(Failure::UnknownCommand(subcommand)),
    }

    Ok(())
}
