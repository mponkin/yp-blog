use std::{fs, path::Path};

use blog_client::{Transport, blog_client::BlogClient, error::BlogClientError};
use clap::Parser;

use crate::{
    cli::{Cli, Command},
    error::CliError,
};
mod cli;
mod error;

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let args = Cli::parse();
    let transport = get_transport(args.grpc, &args.server);

    let client = BlogClient::new(transport).await?;

    let result = handle_command(client, args.command).await;

    match result {
        Ok(message) => println!("OK: {message}"),
        Err(e) => {
            if is_token_invalid(&e) {
                eprintln!("Token is invalid, authorization required for next use");
                delete_token()?;
            }
            return Err(e);
        }
    }

    Ok(())
}

async fn handle_command(mut client: BlogClient, command: Command) -> Result<String, CliError> {
    match command {
        cli::Command::Register {
            username,
            email,
            password,
        } => {
            let token = client.register(username, email, password).await?;
            save_token(token)?;
            Ok(String::from("User registered succesfully"))
        }
        cli::Command::Login { username, password } => {
            let token = client.login(username, password).await?;
            save_token(token)?;
            Ok(String::from("User logged in succesfully"))
        }
        cli::Command::Create { title, content } => {
            let token = load_token()?;
            client.set_token(token);
            let post = client.create_post(title, content).await?;
            Ok(format!("Created post: {post:?}"))
        }
        cli::Command::Get { id } => {
            let post = client.get_post(id).await?;
            Ok(format!("Got post: {post:?}"))
        }
        cli::Command::Update { id, title, content } => {
            let token = load_token()?;
            client.set_token(token);
            let post = client.update_post(id, title, content).await?;
            Ok(format!("Updated post: {post:?}"))
        }
        cli::Command::Delete { id } => {
            let token = load_token()?;
            client.set_token(token);
            client.delete_post(id).await?;
            Ok(format!("Deleted post with id: {id}"))
        }
        cli::Command::List { limit, offset } => {
            let collection = client.get_posts(limit, offset).await?;
            Ok(format!(
                "Posts offset {} from {}, limit {},\n{}",
                collection.offset,
                collection.total_posts,
                collection.limit,
                collection
                    .posts
                    .iter()
                    .map(|p| format!("* {p:?}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        }
        cli::Command::Logout => {
            delete_token()?;
            Ok("User logged out".to_string())
        }
    }
}

const TOKEN_FILE: &str = ".blog_token";

fn save_token(token: String) -> Result<(), CliError> {
    fs::write(TOKEN_FILE, token)?;
    Ok(())
}

fn load_token() -> Result<String, CliError> {
    let path = Path::new(TOKEN_FILE);

    if path.exists() {
        let token = fs::read_to_string(path)?.trim().to_string();

        if token.is_empty() {
            Err(CliError::TokenNotFound)
        } else {
            Ok(token)
        }
    } else {
        Err(CliError::TokenNotFound)
    }
}

fn delete_token() -> Result<(), CliError> {
    let path = Path::new(TOKEN_FILE);

    if path.exists() {
        fs::remove_file(path)?;
    }

    Ok(())
}

fn get_transport(grpc: bool, server: &Option<String>) -> Transport {
    const DEFAULT_ADDRESS: &str = "http://127.0.0.1";
    const DEFAULT_HTTP_PORT: &str = "8080";
    const DEFAULT_GRPC_PORT: &str = "50051";

    if grpc {
        Transport::Grpc(
            server
                .clone()
                .unwrap_or(format!("{DEFAULT_ADDRESS}:{DEFAULT_GRPC_PORT}")),
        )
    } else {
        Transport::Http(
            server
                .clone()
                .unwrap_or(format!("{DEFAULT_ADDRESS}:{DEFAULT_HTTP_PORT}")),
        )
    }
}

fn is_token_invalid(error: &CliError) -> bool {
    matches!(
        error,
        CliError::ClientError(BlogClientError::InvalidToken)
            | CliError::ClientError(BlogClientError::InvalidCredentials)
    )
}
