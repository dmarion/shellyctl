use reqwest::Client;
use serde_json::json;
use std::fs;

pub async fn handle(device: String, slot: u8, file: String) -> anyhow::Result<()> {
    let client = Client::new();
    let code = fs::read_to_string(&file)?;

    let url = format!("http://{}/rpc/Script.PutCode", device);
    let payload = json!({"id": slot, "code": code});
    let res = client.post(&url).json(&payload).send().await?;

    if !res.status().is_success() {
        anyhow::bail!("Upload failed: {}", res.status());
    }

    println!("✅ Uploaded to slot {}", slot);

    let status_url = format!("http://{}/rpc/Script.GetStatus", device);
    let status_res = client.post(&status_url).json(&json!({"id": slot})).send().await?;
    let json: serde_json::Value = status_res.json().await?;

    if json["running"].as_bool().unwrap_or(false) {
        println!("🔄 Script already enabled");
    } else {
        println!("🟢 Enabling script...");
        let enable_url = format!("http://{}/rpc/Script.Enable", device);
        client.post(&enable_url).json(&json!({"id": slot})).send().await?;
        println!("✅ Script enabled");
    }

    Ok(())
}
