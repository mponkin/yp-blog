use clap::{Parser, Subcommand, command};

#[derive(Debug, Parser)]
#[command(author, version, about = "Blog CLI Tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long)]
    pub grpc: bool,

    #[arg(long)]
    pub server: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    Logout,
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    Get {
        #[arg(long)]
        id: i64,
    },
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    Delete {
        #[arg(long)]
        id: i64,
    },
    List {
        #[arg(long)]
        limit: Option<u64>,
        #[arg(long)]
        offset: Option<u64>,
    },
}
