use reqwest::Error;
use serde::Deserialize;

#[derive(Deserialize)]
struct Directive {
    id: String,
    command: String,
}


pub async fn get_directives() -> Result<Vec<String>, Error> {
    let url = "https://127.0.0.1:3030/directives";

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)  
        .build()?;

    let response = client.get(url).send().await?.error_for_status()?;

    let directives: Vec<Directive> = response.json().await?;

    let commands: Vec<String> = directives.into_iter()
        .map(|directive| directive.command)
        .collect();
    
    Ok(commands)
}
