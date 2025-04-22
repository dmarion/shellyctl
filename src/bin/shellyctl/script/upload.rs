use crate::cli::UploadScriptArgs;
use log::{debug, error, info, warn};
use reqwest::Client;
use serde_json::json;

pub async fn handle(args: UploadScriptArgs) -> anyhow::Result<()> {
    let client = Client::new();
    let code = std::fs::read_to_string(&args.file)?;
    debug!("Read script from file: {}", args.file);

    // 1. Call Script.List to find the script by name
    let list_url = format!("http://{}/rpc/Script.List", args.device);
    let list_res = client.get(&list_url).send().await?;
    let list_json: serde_json::Value = list_res.json().await?;
    debug!("Script.List response: {:?}", list_json);

    let scripts = list_json
        .get("scripts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'scripts' array in Script.List"))?;

    // 2. Search for the script by name
    let matching_script = scripts
        .iter()
        .find(|s| s.get("name") == Some(&json!(args.name)));

    let script_id: u8;

    if let Some(existing) = matching_script {
        script_id = existing["id"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Invalid script ID in Script.List response"))?
            as u8;
        debug!("Found script '{}' with ID {}", args.name, script_id);

        if existing["running"].as_bool().unwrap_or(false) && args.force {
            info!(
                "Stopping running script '{}' (ID {})...",
                args.name, script_id
            );
            let stop_url = format!("http://{}/rpc/Script.Stop", args.device);
            client
                .post(&stop_url)
                .json(&json!({ "id": script_id }))
                .send()
                .await?;
            info!("Script stopped");
        } else if existing["running"].as_bool().unwrap_or(false) {
            warn!(
                "Script '{}' is running. Use --force to stop and overwrite.",
                args.name
            );
            return Ok(());
        }
    } else {
        // 3. Script not found → Create it
        info!("Script '{}' not found. Creating it...", args.name);
        let create_url = format!("http://{}/rpc/Script.Create", args.device);
        let create_res = client
            .post(&create_url)
            .json(&json!({ "name": args.name }))
            .send()
            .await?;

        let create_json: serde_json::Value = create_res.json().await?;
        debug!("Script.Create response: {:?}", create_json);

        script_id = create_json["id"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Missing 'id' in Script.Create response"))?
            as u8;
        info!("Created script '{}' with ID {}", args.name, script_id);
    }

    // 4. Upload code
    let put_url = format!("http://{}/rpc/Script.PutCode", args.device);
    let payload = json!({ "id": script_id, "code": code });
    debug!(
        "Uploading code to script ID {}: {}",
        script_id,
        serde_json::to_string_pretty(&payload)
            .unwrap_or_else(|e| format!("<< failed to serialize JSON: {} >>", e))
    );

    let put_res = client.post(&put_url).json(&payload).send().await?;
    if !put_res.status().is_success() {
        let status = put_res.status();
        let body = put_res.text().await.unwrap_or_default();
        error!("Upload failed: {}", body);
        anyhow::bail!("Script.PutCode failed: {}", status);
    }

    info!("Uploaded code to script '{}'", args.name);

    // 5. Optionally enable script
    if args.enable {
        let status_url = format!("http://{}/rpc/Script.GetStatus", args.device);
        let status_res = client
            .post(&status_url)
            .json(&json!({ "id": script_id }))
            .send()
            .await?;
        let status_json: serde_json::Value = status_res.json().await?;

        if status_json["running"].as_bool().unwrap_or(false) {
            info!("Script '{}' is already enabled", args.name);
        } else {
            info!("Enabling script '{}'...", args.name);
            let enable_url = format!("http://{}/rpc/Script.Enable", args.device);
            client
                .post(&enable_url)
                .json(&json!({ "id": script_id }))
                .send()
                .await?;
            info!("Script enabled");
        }
    } else {
        debug!("Skipping script enable (--enable not passed)");
    }

    Ok(())
}
