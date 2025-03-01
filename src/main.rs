use reqwest::Error;

mod connexion;

#[tokio::main]
async fn main() -> Result<(), Error> {
    match connexion::get_directives().await {
        Ok(commands) => {
            println!("Commandes récupérées :");
            for command in &*commands {
                println!("{}", command);
            }
        },

        Err(e) => eprintln!("Erreur avec la connexion au C2: {}", e),
    }
    Ok(())
}