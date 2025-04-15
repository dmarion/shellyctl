use anyhow::{bail, Result};
use clap::Args;
use colored::*;
use reqwest::Client;
use serde_json::Value;

#[derive(Args)]
pub struct ConfigDumpArgs {
    /// Device IP or hostname
    #[arg(short, long)]
    pub device: String,

    /// Optional key path to print a subtree, e.g. .wifi.ap
    #[arg(long)]
    pub subtree: Option<String>,
}

pub async fn handle(args: ConfigDumpArgs) -> Result<()> {
    let url = format!("http://{}/rpc/Shelly.GetConfig", args.device);
    let client = Client::new();

    let resp = client
        .post(&url)
        .json(&serde_json::json!({}))
        .send()
        .await?;

    let mut data: Value = resp.json().await?;

    if let Some(subtree_path) = &args.subtree {
        let path = subtree_path.trim_start_matches('.').split('.');
        for key in path {
            match data.get(key) {
                Some(sub) => data = sub.clone(),
                None => bail!("Subtree path '{}' not found", subtree_path),
            }
        }
    }

    print_tree(&data, 0);
    Ok(())
}

fn print_tree(value: &Value, indent: usize) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                print_indent(indent);
                if v.is_object() || v.is_array() {
                    println!("{}:", k.white());
                    print_tree(v, indent + 2);
                } else {
                    println!("{}: {}", k.white(), format_leaf(v));
                }
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                print_indent(indent);
                println!("[{}]", i);
                print_tree(v, indent + 2);
            }
        }
        _ => {
            print_indent(indent);
            println!("{}", format_leaf(value));
        }
    }
}

fn print_indent(n: usize) {
    print!("{}", " ".repeat(n));
}

fn format_leaf(value: &Value) -> impl std::fmt::Display {
    match value {
        Value::String(s) => s.green(),
        Value::Number(n) => n.to_string().yellow(),
        Value::Bool(b) => format!("{}", b).cyan(),
        Value::Null => "null".italic().dimmed(),
        _ => value.to_string().normal(),
    }
}
