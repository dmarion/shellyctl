use crate::cli::ListScriptsArgs;
use prettytable::{
    format::{FormatBuilder, LinePosition, LineSeparator},
    Cell, Row, Table,
};
use reqwest::Client;
use serde_json::Value;

pub async fn handle(args: ListScriptsArgs) -> anyhow::Result<()> {
    let client = Client::new();
    let url = format!("http://{}/rpc/Script.List", args.device);
    let res = client.get(&url).send().await?;

    if !res.status().is_success() {
        anyhow::bail!("List request failed: {}", res.status());
    }

    let json: Value = res.json().await?;
    let scripts = json["scripts"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Invalid response format: 'scripts' is not an array"))?;

    if scripts.is_empty() {
        println!("No scripts found.");
        return Ok(());
    }

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

    // Header row with style_spec
    table.add_row(Row::new(vec![
        Cell::new("ID").style_spec("Fc"),
        Cell::new("Name").style_spec("Fc"),
        Cell::new("Running").style_spec("Fc"),
        Cell::new("AutoStart").style_spec("Fc"),
    ]));

    for script in scripts {
        let id = script["id"].as_u64().unwrap_or(0);
        let name = script["name"].as_str().unwrap_or("<unknown>");
        let running = script["running"].as_bool().unwrap_or(false);
        let autostart = script["autostart"].as_bool().unwrap_or(false);

        table.add_row(Row::new(vec![
            Cell::new(&id.to_string()).style_spec("Fg"),      // Green
            Cell::new(name).style_spec("Fw"),                 // White
            Cell::new(&running.to_string()).style_spec("Fy"), // Yellow
            Cell::new(&autostart.to_string()).style_spec("Fy"), // Yellow
        ]));
    }

    table.printstd();
    Ok(())
}
