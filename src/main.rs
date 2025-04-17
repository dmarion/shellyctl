mod browse;
mod cli;
mod configdump;
mod configset;
mod download;
mod list;
mod upload;

use clap::Parser;
use cli::{Cli, Commands};
use std::sync::atomic::{AtomicU8, Ordering};

static VERBOSITY: AtomicU8 = AtomicU8::new(0);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    VERBOSITY.store(cli.verbose, Ordering::Relaxed);

    match cli.command {
        Commands::Upload { device, slot, file } => upload::handle(device, slot, file).await?,
        Commands::Download { device, slot, file } => download::handle(device, slot, file).await?,
        Commands::List { device } => list::handle(device).await?,
        Commands::Browse(args) => browse::handle(args).await?,
        Commands::ConfigSet(args) => configset::handle(args).await?,
        Commands::ConfigDump(args) => configdump::handle(args).await?,
    }
    Ok(())
}

pub fn log_verbose(message: &str) {
    if VERBOSITY.load(Ordering::Relaxed) > 0 {
        println!("[rpc] {}", message);
    }
}
