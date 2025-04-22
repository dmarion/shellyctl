use reqwest::Client;

pub async fn handle(device: String) -> anyhow::Result<()> {
    let client = Client::new();
    let url = format!("http://{}/rpc/Script.List", device);
    let res = client.get(&url).send().await?;

    if !res.status().is_success() {
        anyhow::bail!("List request failed: {}", res.status());
    }

    let body = res.text().await?;
    println!("{}", body);
    Ok(())
}
