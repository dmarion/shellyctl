mod browse;
mod cli;
mod configdump;
mod configset;
mod download;
mod list;
mod upload;

use clap::Parser;
use cli::{Cli, Commands, ConfigCommand, ScriptCommand};
use std::sync::atomic::{AtomicU8, Ordering};

static VERBOSITY: AtomicU8 = AtomicU8::new(0);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    VERBOSITY.store(cli.verbose, Ordering::Relaxed);

    match cli.command {
        Commands::Script { command } => match command {
            ScriptCommand::Upload(args) => {
                upload::handle(args.device, args.slot, args.file).await?
            }
            ScriptCommand::Download(args) => {
                download::handle(args.device, args.slot, args.file).await?
            }
            ScriptCommand::List(args) => list::handle(args.device).await?,
        },
        Commands::Config { command } => match command {
            ConfigCommand::Set(args) => configset::handle(args).await?,
            ConfigCommand::Dump(args) => configdump::handle(args).await?,
        },
        Commands::Browse(args) => browse::handle(args).await?,
    }
    Ok(())
}

pub fn log_verbose(message: &str) {
    if VERBOSITY.load(Ordering::Relaxed) > 0 {
        println!("[rpc] {}", message);
    }
}
