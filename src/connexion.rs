use reqwest::Error;
use serde::Deserialize;
use std::ops::Deref;

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Directive(Vec<String>);

impl Deref for Directive {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn get_directives() -> Result<Directive, Error> {
    let url = "https://127.0.0.1:3030/directives";

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)  
        .build()?;

    let response = client.get(url).send().await?.error_for_status()?;

    let commands: Directive = response.json().await?;
    
    Ok(commands)
}
