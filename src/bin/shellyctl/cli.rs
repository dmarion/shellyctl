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

    #[arg(short, long, help = "Script name")]
    pub name: String,

    #[arg(short, long, help = "File to save downloaded script")]
    pub file: Option<String>,

    #[arg(long, help = "Print script code to stdout instead of saving")]
    pub stdout: bool,

    #[arg(short = 'y', long, help = "Overwrite file without confirmation")]
    pub yes: bool,
}

#[derive(Args)]
pub struct UploadScriptArgs {
    #[arg(short, long, help = "Device IP or hostname", alias = "d")]
    pub device: String,

    #[arg(short, long, help = "Script name", alias = "n")]
    pub name: String,

    #[arg(short, long, help = "Script file to upload", alias = "f")]
    pub file: String,

    /// Force overwrite the script even if one exists or is running
    #[arg(long)]
    pub force: bool,

    /// Enable script after upload
    #[arg(short, long)]
    pub enable: bool,
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
