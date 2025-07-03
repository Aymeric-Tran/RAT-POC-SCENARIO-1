use reqwest::StatusCode;
use std::time::Duration;

pub async fn check_ks() -> bool {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return true,
    };

    let response = match client
        .get("http://iuqerfsodp9ifjaposqqqqqqqdfjhgosurijfaewrwergwea.com")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => {
            println!("Kill switch désactivé (erreur réseau)");
            return false;
        }
    };

    if response.status() == StatusCode::OK {
        println!("KILL SWITCH ACTIVÉ: Le domaine existe et retourne 200 OK");
        true
    } else {
        println!("Kill switch désactivé. Statut HTTP: {}", response.status());
        false
    }
}
