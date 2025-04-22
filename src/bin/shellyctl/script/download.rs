use crate::cli::DownloadScriptArgs;
use anyhow::{anyhow, bail};
use log::{debug, error, info, warn};
use reqwest::Client;
use serde_json::{json, Value};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub async fn handle(args: DownloadScriptArgs) -> anyhow::Result<()> {
    let client = Client::new();

    // 1. Fetch all scripts
    let list_url = format!("http://{}/rpc/Script.List", args.device);
    debug!("Requesting Script.List from {}", list_url);
    let list_res = client.get(&list_url).send().await?;
    if !list_res.status().is_success() {
        error!("Failed to fetch script list: {}", list_res.status());
        bail!("Failed to fetch script list: {}", list_res.status());
    }

    let list_json: Value = list_res.json().await?;
    let scripts = list_json["scripts"]
        .as_array()
        .ok_or_else(|| anyhow!("Invalid response: missing 'scripts' array"))?;

    // 2. Find script by name
    let script = scripts.iter().find(|s| s["name"] == args.name);
    let script = match script {
        Some(script) => script,
        None => {
            error!("Script '{}' not found on device {}", args.name, args.device);
            bail!("Script '{}' not found on device {}", args.name, args.device);
        }
    };

    let id = script["id"]
        .as_u64()
        .ok_or_else(|| anyhow!("Missing or invalid script ID"))? as u8;
    debug!("Resolved script '{}' to ID {}", args.name, id);

    // 3. Get the script code
    let get_url = format!("http://{}/rpc/Script.GetCode", args.device);
    debug!("Requesting Script.GetCode for ID {}", id);
    let get_res = client
        .post(&get_url)
        .json(&json!({ "id": id }))
        .send()
        .await?;
    if !get_res.status().is_success() {
        error!("Failed to fetch script code: {}", get_res.status());
        bail!("Failed to fetch script code: {}", get_res.status());
    }

    let get_json: Value = get_res.json().await?;
    let code = get_json["data"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing or invalid 'data' field"))?;

    // 4. Output to stdout
    if args.stdout {
        println!("{}", code);
        return Ok(());
    }

    // 5. Determine filename
    let filename = args
        .file
        .clone()
        .unwrap_or_else(|| generate_safe_filename(&args.name));
    debug!("Output file: {}", filename);

    // 6. Check for file overwrite
    if Path::new(&filename).exists() && !args.yes {
        warn!("File '{}' already exists.", filename);
        print!("Overwrite? [y/N]: ");
        io::stdout().flush()?;
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        if !matches!(response.trim().to_lowercase().as_str(), "y" | "yes") {
            warn!("Cancelled by user.");
            return Ok(());
        }
    }

    // 7. Save to file
    fs::write(&filename, code)?;
    info!("✅ Script saved to '{}'", filename);

    Ok(())
}

// Helper to generate safe filename from script name
fn generate_safe_filename(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    format!("{}.js", sanitized)
}
