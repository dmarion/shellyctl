use reqwest::Client;
use serde_json::json;
use std::fs;

pub async fn handle(device: String, slot: u8, file: String) -> anyhow::Result<()> {
    let client = Client::new();
    let url = format!("http://{}/rpc/Script.GetCode", device);
    let res = client.post(&url).json(&json!({"id": slot})).send().await?;

    if !res.status().is_success() {
        anyhow::bail!("Download failed: {}", res.status());
    }

    let json: serde_json::Value = res.json().await?;
    let code = json["code"].as_str().unwrap_or("");
    fs::write(&file, code)?;

    println!("✅ Saved script to {}", file);
    Ok(())
}
