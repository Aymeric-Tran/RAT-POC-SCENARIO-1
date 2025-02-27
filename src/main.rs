use reqwest::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Directive(Vec<String>);

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://127.0.0.1:3030/directives";

    let client = reqwest::Client::builder()
    .danger_accept_invalid_certs(true)
    .build()?;

    let response = client.get(url).send().await?;

    if response.status().is_success() {
        let commands: Directive = response.json().await?;
        for command in commands.0.iter() {
            println!("{}", command)
        }
    } else {
        println!("Error {}", response.status());
    }

    Ok(())
}