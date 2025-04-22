mod browse;
mod cli;
mod config {
    pub mod dump;
    pub mod set;
}
mod script {
    pub mod download;
    pub mod list;
    pub mod upload;
}

use clap::Parser;
use cli::{Cli, Commands, ConfigCommand, ScriptCommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if cli.verbose > 0 {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug) // or Info
            .init();
    }

    match cli.command {
        Commands::Script { command } => match command {
            ScriptCommand::Upload(args) => script::upload::handle(args).await?,
            ScriptCommand::Download(args) => script::download::handle(args).await?,
            ScriptCommand::List(args) => script::list::handle(args).await?,
        },
        Commands::Config { command } => match command {
            ConfigCommand::Set(args) => config::set::handle(args).await?,
            ConfigCommand::Dump(args) => config::dump::handle(args).await?,
        },
        Commands::Browse(args) => browse::handle(args).await?,
    }
    Ok(())
}
