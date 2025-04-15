use std::collections::{BTreeMap, HashSet};
use std::io::{stdout, Write};
use std::net::IpAddr;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Args;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use reqwest::Client;
use serde_json::Value;
use tokio::time::sleep;

use crossterm::{
    cursor::MoveUp,
    execute,
    terminal::{Clear, ClearType},
};
use prettytable::{
    format::{FormatBuilder, LinePosition, LineSeparator},
    Cell, Row, Table,
};

#[derive(Args)]
pub struct BrowseArgs {
    #[arg(long, help = "Optional filter by device type (e.g. pro3em)")]
    r#type: Option<String>,
}

#[derive(Debug, Clone)]
struct ShellyDevice {
    mac: String,
    ip: String,
    device_type: String,
    name: String,
    ver: String,
    model: String,
    app: String,
    profile: String,
}

pub async fn handle(args: BrowseArgs) -> Result<()> {
    let mdns = ServiceDaemon::new()?;
    let service_type = "_http._tcp.local.";
    let receiver = mdns.browse(service_type)?;

    println!("🔍 Scanning for Shelly devices on the network (5s)...\n");

    let allowed_types: Option<HashSet<String>> = args
        .r#type
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect());

    let client = Client::new();
    let mut seen = HashSet::<IpAddr>::new();
    let mut devices = BTreeMap::<String, ShellyDevice>::new();

    let start = Instant::now();
    let timeout = Duration::from_secs(5);

    let mut last_height = 0;
    let mut stdout = stdout();

    while start.elapsed() < timeout {
        while let Ok(event) = receiver.try_recv() {
            if let ServiceEvent::ServiceResolved(info) = event {
                if let Some(ip) = info.get_addresses().iter().find(|a| a.is_ipv4()) {
                    if seen.contains(ip) {
                        continue;
                    }
                    seen.insert(*ip);
                    let ip_str = ip.to_string();

                    let url = format!("http://{}/rpc/Shelly.GetDeviceInfo", ip_str);
                    if let Ok(resp) = client
                        .get(&url)
                        .timeout(Duration::from_secs(2))
                        .send()
                        .await
                    {
                        if let Ok(json) = resp.json::<Value>().await {
                            let device_type = json["id"]
                                .as_str()
                                .unwrap_or("-")
                                .to_string()
                                .strip_prefix("shelly")
                                .and_then(|s| s.split_once('-'))
                                .map(|(model, _)| model)
                                .unwrap_or("unknown")
                                .to_string();

                            if let Some(ref allowed) = allowed_types {
                                if !allowed.contains(&device_type) {
                                    continue;
                                }
                            }
                            let mac = json["mac"].as_str().unwrap_or("-").to_string();
                            let get = |key: &str| {
                                json.get(key)
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("-")
                                    .to_string()
                            };

                            devices.insert(
                                mac.clone(),
                                ShellyDevice {
                                    mac,
                                    ip: ip_str,
                                    device_type,
                                    name: get("name"),
                                    ver: get("ver"),
                                    model: get("model"),
                                    app: get("app"),
                                    profile: get("profile"),
                                },
                            );

                            // Prepare table
                            let mut table = Table::new();
                            let format = FormatBuilder::new()
                                .column_separator(' ')
                                .borders('\0')
                                .separator(
                                    LinePosition::Top,
                                    LineSeparator::new('\0', '\0', '\0', '\0'),
                                )
                                .separator(
                                    LinePosition::Title,
                                    LineSeparator::new('\0', '\0', '\0', '\0'),
                                )
                                .separator(
                                    LinePosition::Bottom,
                                    LineSeparator::new('\0', '\0', '\0', '\0'),
                                )
                                .padding(0, 0)
                                .build();
                            table.set_format(format);

                            table.add_row(Row::new(vec![
                                Cell::new("MAC Addr").style_spec("Fc"),
                                Cell::new("IP Addr").style_spec("Fc"),
                                Cell::new("Type").style_spec("Fc"),
                                Cell::new("Model").style_spec("Fc"),
                                Cell::new("App").style_spec("Fc"),
                                Cell::new("Profile").style_spec("Fc"),
                                Cell::new("Firmware").style_spec("Fc"),
                                Cell::new("Name").style_spec("Fc"),
                            ]));

                            for device in devices.values() {
                                table.add_row(Row::new(vec![
                                    Cell::new(&device.mac).style_spec("Fg"),
                                    Cell::new(&device.ip).style_spec("Fw"),
                                    Cell::new(&device.device_type).style_spec("Fy"),
                                    Cell::new(&device.model).style_spec("Fw"),
                                    Cell::new(&device.app).style_spec("Fy"),
                                    Cell::new(&device.profile).style_spec("Fw"),
                                    Cell::new(&device.ver).style_spec("Fy"),
                                    Cell::new(&device.name).style_spec("Fw"),
                                ]));
                            }

                            let height = table.to_string().lines().count();

                            if height > last_height {
                                for _ in 0..(height - last_height) {
                                    println!();
                                }
                                last_height = height;
                            }

                            execute!(
                                stdout,
                                MoveUp(height as u16),
                                Clear(ClearType::FromCursorDown)
                            )?;

                            table.printstd();
                            stdout.flush()?;
                        }
                    }
                }
            }
        }
        sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}
