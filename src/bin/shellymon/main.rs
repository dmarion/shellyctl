use chrono::Utc;
use clap::Parser;
use log::{debug, error, info, warn};
use reqwest::blocking::Client;
use rusqlite::{Connection, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "shellymon")]
struct Cli {
    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    config: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    devices: Vec<DeviceConfig>,
}

#[derive(Debug, Deserialize, Clone)]
struct DeviceConfig {
    name: String,
    address: String,
    db_path: String,
    table: String,
    interval: u64,
    fields: HashMap<String, String>,
}

fn ensure_table(conn: &Connection, table: &str, fields: &HashMap<String, String>) -> Result<()> {
    let mut columns: Vec<String> = vec!["timestamp TEXT".into(), "device TEXT".into()];
    for column_name in fields.values() {
        columns.push(format!("{} REAL", column_name));
    }
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS {} ({});",
        table,
        columns.join(", ")
    );
    conn.execute(&sql, [])?;
    Ok(())
}

fn ensure_column(conn: &Connection, table: &str, column_name: &str) {
    let check_sql = format!("PRAGMA table_info({});", table);
    let mut stmt = conn.prepare(&check_sql).unwrap();
    let existing: Vec<String> = stmt
        .query_map([], |row| row.get(1))
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    if !existing.contains(&column_name.to_string()) {
        let alter_sql = format!("ALTER TABLE {} ADD COLUMN {} REAL;", table, column_name);
        if let Err(e) = conn.execute(&alter_sql, []) {
            error!("Failed to add column {} to {}: {}", column_name, table, e);
        } else {
            debug!("Added column '{}' to table '{}'.", column_name, table);
        }
    }
}

fn load_config(path: &str) -> Config {
    info!("Loading configuration from: {}", path);
    let config_data = fs::read_to_string(path).expect("Failed to read config file");
    toml::from_str(&config_data).expect("Failed to parse config")
}

fn extract_json_value(json: &serde_json::Value, path: &str) -> Option<f64> {
    let mut current = json;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    current.as_f64()
}

fn run_monitoring_loop(devices: &[DeviceConfig], running: Arc<AtomicBool>) {
    let mut last_run: HashMap<String, Instant> = HashMap::new();
    let client = Client::new();

    info!(
        "Starting monitoring loop for {} device(s)...",
        devices.len()
    );

    while running.load(Ordering::Relaxed) {
        for device in devices {
            let now = Instant::now();
            let last = last_run
                .get(&device.name)
                .cloned()
                .unwrap_or_else(|| now - Duration::from_secs(device.interval));

            if now.duration_since(last).as_secs() >= device.interval {
                let url = format!("http://{}/rpc/Shelly.GetStatus", device.address);
                info!("Polling {} at {}", device.name, url);

                match client.get(&url).send() {
                    Ok(resp) => {
                        if let Ok(json) = resp.json::<serde_json::Value>() {
                            let timestamp = Utc::now().to_rfc3339();
                            if let Ok(conn) = Connection::open(&device.db_path) {
                                if let Err(e) = ensure_table(&conn, &device.table, &device.fields) {
                                    error!("Failed to ensure table '{}': {}", device.table, e);
                                    continue;
                                }

                                let mut sql =
                                    format!("INSERT INTO {} (timestamp, device", device.table);
                                let mut placeholders = String::from("?, ?");
                                let mut raw_values: Vec<f64> = Vec::new();
                                let mut values: Vec<&dyn rusqlite::ToSql> =
                                    vec![&timestamp, &device.name];

                                for (json_path, column_name) in &device.fields {
                                    ensure_column(&conn, &device.table, column_name);

                                    if let Some(val) = extract_json_value(&json, json_path) {
                                        sql.push_str(&format!(", {}", column_name));
                                        placeholders.push_str(", ?");
                                        raw_values.push(val);
                                    } else {
                                        warn!(
                                            "Missing or invalid value for '{}' on device '{}'",
                                            json_path, device.name
                                        );
                                    }
                                }

                                for val in &raw_values {
                                    values.push(val);
                                }

                                sql.push_str(&format!(") VALUES ({});", placeholders));

                                if let Err(e) = conn.execute(&sql, values.as_slice()) {
                                    error!("Insert error for {}: {}", device.name, e);
                                } else {
                                    info!(
                                        "Logged data for device '{}' into table '{}'.",
                                        device.name, device.table
                                    );
                                }
                            } else {
                                error!("Failed to open database for device '{}'.", device.name);
                            }
                        }
                    }
                    Err(e) => {
                        error!("HTTP error for device '{}': {}", device.name, e);
                    }
                }

                last_run.insert(device.name.clone(), now);
            }
        }

        std::thread::sleep(Duration::from_secs(1));
    }

    info!("Graceful shutdown complete.");
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::init();
    }

    let config = load_config(&cli.config);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl+C handler");

    run_monitoring_loop(&config.devices, running);

    Ok(())
}
