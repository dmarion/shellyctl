mod browse;
mod configdump;
mod configset;
mod download;
mod list;
mod upload;

use clap::{Parser, Subcommand};
use std::sync::atomic::{AtomicU8, Ordering};

static VERBOSITY: AtomicU8 = AtomicU8::new(0);

#[derive(Parser)]
#[command(name = "shellyctl")]
#[command(about = "Control Shelly Gen2+ devices", long_about = None)]
struct Cli {
    /// Increase verbosity (-v, -vv, etc.)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Upload {
        #[arg(short, long, help = "Device IP or hostname", alias = "d")]
        device: String,

        #[arg(
            short,
            long,
            default_value_t = 0,
            help = "Script slot ID (default: 0)",
            alias = "s"
        )]
        slot: u8,

        #[arg(help = "Script file to upload")]
        file: String,
    },
    Download {
        #[arg(short, long, help = "Device IP or hostname", alias = "d")]
        device: String,

        #[arg(
            short,
            long,
            default_value_t = 0,
            help = "Script slot ID (default: 0)",
            alias = "s"
        )]
        slot: u8,

        #[arg(help = "File to save downloaded script")]
        file: String,
    },
    List {
        #[arg(short, long, help = "Device IP or hostname", alias = "d")]
        device: String,
    },
    Browse(browse::BrowseArgs),
    ConfigSet(configset::ConfigSetArgs),
    ConfigDump(configdump::ConfigDumpArgs),
}

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
