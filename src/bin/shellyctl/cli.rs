use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "shellyctl")]
#[command(about = "Control Shelly Gen2+ devices", long_about = None)]
pub struct Cli {
    /// Increase verbosity (-v, -vv, etc.)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
    Browse(BrowseArgs),
    ConfigSet(ConfigSetArgs),

    /// Dump full or partial configuration tree
    #[command(name = "config-dump")]
    ConfigDump(ConfigDumpArgs),
}

#[derive(Args)]
pub struct BrowseArgs {
    #[arg(long, help = "Optional filter by device type (e.g. pro3em)")]
    pub r#type: Option<String>,
}

#[derive(Args)]
pub struct ConfigSetArgs {
    /// Device IP or hostname
    #[arg(short, long)]
    pub device: String,

    /// Key-value pairs to modify, e.g. Sys.device.name=klima
    #[arg(required = true)]
    pub pairs: Vec<String>,
}

#[derive(Args)]
pub struct ConfigDumpArgs {
    /// Device IP or hostname
    #[arg(short, long)]
    pub device: String,

    /// Optional key path to print a subtree, e.g. .wifi.ap
    #[arg(long)]
    pub subtree: Option<String>,
}
