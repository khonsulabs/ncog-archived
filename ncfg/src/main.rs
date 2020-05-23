use gumdrop::Options;
use ncog_client::{LoginState, Network, NetworkEvent};
use shared::ServerRequest;

#[derive(Debug, Options)]
struct NcfgOptions {
    #[options(help = "run against production")]
    help: bool,
    #[options(help = "run against production")]
    production: bool,
    #[options(command)]
    command: Option<Command>,
}

#[derive(Debug, Options)]
enum Command {
    #[options(help = "force authentication to run")]
    Auth(AuthOptions),
}

#[derive(Debug, Options)]
struct AuthOptions {}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = NcfgOptions::parse_args_default_or_exit();

    if let Some(command) = &opts.command {
        match command {
            Command::Auth(_) => authenticate().await,
        }
    } else {
        println!("Available commands:");
        println!("{}", NcfgOptions::command_list().unwrap());
        println!("{}", NcfgOptions::usage());

        Ok(())
    }
}

async fn authenticate() -> Result<(), anyhow::Error> {
    let event_receiver = Network::event_receiver().await;
    println!("Connecting...");
    Network::spawn("ws://localhost:7878").await;

    loop {
        match event_receiver.recv()? {
            NetworkEvent::LoginStateChanged => match Network::login_state().await {
                LoginState::LoggedOut => unreachable!(),
                LoginState::Error { message } => {
                    println!("Error with network {:?}", message);
                }
                LoginState::Connected => {
                    println!("Connection established.");
                    Network::request(ServerRequest::AuthenticationUrl).await;
                }
                LoginState::Authenticated { profile } => {
                    println!("Sucessfully logged in: {:?}", profile);
                    break;
                }
            },
            NetworkEvent::AuthenticateAtUrl(url) => {
                println!("Open this URL to log in: {}", url);
            }
        }
    }

    Ok(())
}
