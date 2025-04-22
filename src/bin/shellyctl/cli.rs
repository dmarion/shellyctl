use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Increase verbosity (-v, -vv, etc.)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(visible_alias = "sc")]
    Script {
        #[command(subcommand)]
        command: ScriptCommand,
    },

    #[command(visible_alias = "cfg")]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },

    Browse(BrowseArgs),
}

#[derive(Subcommand)]
pub enum ScriptCommand {
    #[command(visible_alias = "dl")]
    Download(DownloadScriptArgs),

    #[command(visible_alias = "up")]
    Upload(UploadScriptArgs),

    #[command(visible_alias = "ls")]
    List(ListScriptsArgs),
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    #[command(name = "set")]
    Set(ConfigSetArgs),

    #[command(name = "dump")]
    Dump(ConfigDumpArgs),
}

#[derive(Args)]
pub struct DownloadScriptArgs {
    #[arg(short, long, help = "Device IP or hostname", alias = "d")]
    pub device: String,

    #[arg(
        short,
        long,
        default_value_t = 0,
        help = "Script slot ID (default: 0)",
        alias = "s"
    )]
    pub slot: u8,

    #[arg(help = "File to save downloaded script")]
    pub file: String,
}

#[derive(Args)]
pub struct UploadScriptArgs {
    #[arg(short, long, help = "Device IP or hostname", alias = "d")]
    pub device: String,

    #[arg(
        short,
        long,
        default_value_t = 0,
        help = "Script slot ID (default: 0)",
        alias = "s"
    )]
    pub slot: u8,

    #[arg(help = "Script file to upload")]
    pub file: String,
}

#[derive(Args)]
pub struct ListScriptsArgs {
    #[arg(short, long, help = "Device IP or hostname", alias = "d")]
    pub device: String,
}

#[derive(Args)]
pub struct BrowseArgs {
    #[arg(long, help = "Filter by device type (comma-separated)")]
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
