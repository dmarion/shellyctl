use crate::cli::ConfigSetArgs;
use crate::log_verbose;
use anyhow::{bail, Result};
use reqwest::Client;
use serde_json::{json, to_string_pretty, Map, Value};

fn parse_value(val: &str) -> Value {
    if val.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if val.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else if let Ok(n) = val.parse::<i64>() {
        Value::Number(n.into())
    } else if let Ok(f) = val.parse::<f64>() {
        Value::Number(serde_json::Number::from_f64(f).unwrap())
    } else {
        Value::String(val.to_string())
    }
}

fn insert_nested_key(config: &mut Map<String, Value>, key: &str, value: &Value) {
    let mut parts = key.trim_start_matches('.').split('.').peekable();
    let mut current = config;
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            current.insert(part.to_string(), value.clone());
        } else {
            current = current
                .entry(part)
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .expect("intermediate key is not an object");
        }
    }
}

pub async fn handle(args: ConfigSetArgs) -> Result<()> {
    let client = Client::new();

    // First, get available methods
    let list_url = format!("http://{}/rpc/Shelly.ListMethods", args.device);
    let method_list: Value = client
        .post(&list_url)
        .json(&json!({}))
        .send()
        .await?
        .json()
        .await?;

    let Some(methods) = method_list.get("methods") else {
        bail!("Missing 'methods' in Shelly.ListMethods response");
    };
    let available_methods: Vec<String> = methods
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|m| m.as_str().map(|s| s.to_string()))
        .collect();

    // Group each KVP independently per RPC
    let mut by_rpc: Vec<(String, String, Value)> = vec![];

    for pair in &args.pairs {
        if let Some((rpc_key, v)) = pair.split_once('=') {
            if let Some((prefix, keypath)) = rpc_key.split_once('.') {
                let rpc = prefix.to_ascii_lowercase();
                let key = keypath.to_string();
                let value = parse_value(v);
                by_rpc.push((rpc, key, value));
            } else {
                eprintln!("❌ Invalid key format (missing dot): {}", pair);
            }
        } else {
            eprintln!("❌ Invalid key=value pair: {}", pair);
        }
    }

    use std::collections::HashMap;
    let mut rpc_configs: HashMap<String, Map<String, Value>> = HashMap::new();
    for (rpc, key, value) in by_rpc {
        let map = rpc_configs.entry(rpc.clone()).or_default();
        insert_nested_key(map, &key, &value);
    }

    for (rpc, config) in rpc_configs {
        // Search for matching methods case-insensitively
        let set_method = available_methods
            .iter()
            .find(|m| m.to_lowercase() == format!("{}.setconfig", rpc))
            .cloned();
        let get_method = available_methods
            .iter()
            .find(|m| m.to_lowercase() == format!("{}.getconfig", rpc))
            .cloned();

        let Some(set_method) = set_method else {
            eprintln!("❌ Method not supported: {}.SetConfig", rpc);
            continue;
        };

        // Optional: validate keys with GetConfig
        if let Some(get_method) = get_method {
            let get_url = format!("http://{}/rpc/{}", args.device, get_method);
            let current_config: Value = client
                .post(&get_url)
                .json(&json!({}))
                .send()
                .await?
                .json()
                .await?;

            log_verbose(&format!(
                "Current {} config:\n{}",
                get_method,
                to_string_pretty(&current_config)?
            ));
        }

        let url = format!("http://{}/rpc/{}", args.device, set_method);
        let body = json!({ "config": config });

        log_verbose(&format!(
            "POST {}\nBODY:\n{}",
            url,
            to_string_pretty(&body)?
        ));

        let resp = client.post(&url).json(&body).send().await?;

        if resp.status().is_success() {
            println!("✅ {} updated on {}", set_method, args.device);
        } else {
            let err = resp.text().await?;
            eprintln!("❌ Failed to update {}: {}", set_method, err);
        }
    }

    Ok(())
}
