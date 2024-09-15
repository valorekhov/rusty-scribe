use anyhow::{Result, Context};
use bytemuck::NoUninit;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SizedSample;
use hound::{WavWriter, WavSpec, SampleFormat};
use std::sync::mpsc::{self, Sender};
use std::time::Duration;
use log::{info, error};

pub fn list_audio_devices() -> Result<()> {
    let host = cpal::default_host();

    println!("Available input audio devices:");
    for device in host.input_devices().context("Failed to get input devices")? {
        println!("{}", device.name().unwrap_or_else(|_| "Unknown".to_string()));
    }
    Ok(())
}

/// Records audio from the specified device for the given duration in seconds
pub fn record_audio(device_name: &str, duration_secs: u64, file_path: &str) -> Result<()> {
    let host = cpal::default_host();

    let device = if device_name.to_lowercase() == "default" {
        host.default_input_device().context("No default input device available")?
    } else {
        host.input_devices()
            .context("Failed to get input devices")?
            .find(|d| d.name().map(|n| n == device_name).unwrap_or(false))
            .context("Specified recording device not found")?
    };

    info!("Using audio device: {}", device.name()?);

    let config = device.default_input_config().context("Failed to get default input config")?;

    let sample_format = config.sample_format();
    let config: cpal::StreamConfig = config.into();

    // Setup WAV writer
    let spec = WavSpec {
        channels: config.channels,
        sample_rate: config.sample_rate.0,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(file_path, spec)
        .with_context(|| format!("Failed to create WAV file at {}", file_path))?;

    // Create a channel to receive audio samples
    let (tx, rx) = mpsc::channel();

    // Build and run the stream
    let stream = match sample_format {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, tx.clone())?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, tx.clone())?,
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config, tx.clone())?,
        _ => return Err(anyhow::anyhow!("Unsupported sample format")),
    };

    stream.play().context("Failed to start audio stream")?;

    info!("Recording audio for {} seconds...", duration_secs);

    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(duration_secs) {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(sample) => {
                writer.write_sample(sample)
                    .context("Failed to write audio sample to WAV")?;
            },
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    
    // Drain any remaining samples
    while let Ok(sample) = rx.try_recv() {
        writer.write_sample(sample)
            .context("Failed to write audio sample to WAV")?;
    }

    drop(stream);
    writer.finalize().context("Failed to finalize WAV file")?;

    info!("Audio recording saved to {}", file_path);
    Ok(())
}
/// Helper function to build an input stream
fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    tx: Sender<i16>,
) -> Result<cpal::Stream>
where
    T: cpal::Sample + NoUninit + SizedSample
{
    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let sample_i16: &[i16] = bytemuck::cast_slice::<T, i16>(data);
            for &sample in sample_i16.iter() {
                if tx.send(sample).is_err() {
                    // Receiver disconnected
                    break;
                }
            }
        },
        move |err| {
            error!("An error occurred on the input stream: {}", err);
        },
        None,
    ).context("Failed to build input stream")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_parse_audio_devices() {
        // This test will list audio devices and ensure the function runs without error.
        // Note: The actual devices depend on the test environment.
        // The test passes if the function completes without panicking.
        let result = list_audio_devices();
        assert!(result.is_ok());
    }
    #[test]
    fn test_record_audio_success() {
        // Record a short audio snippet and ensure the file is created.
        // Note: This test will actually record audio from the default device.
        // It's better to mock the audio input, but for simplicity, we'll perform a real recording.

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_recording.wav");
        let file_path_str = file_path.to_str().unwrap();

        // Increase the duration to ensure we get a complete number of samples
        let result = record_audio("default", 2, file_path_str);
        if let Err(e) = &result {
            eprintln!("Error recording audio: {:?}", e);
        }
        assert!(result.is_ok());

        // Check that the file exists and is not empty
        assert!(file_path.exists());
        let metadata = fs::metadata(&file_path).expect("Failed to get file metadata");
        assert!(metadata.len() > 0);

        // Clean up
        temp_dir.close().expect("Failed to delete temp dir");
    }

    #[test]
    fn test_record_audio_invalid_device() {
        // Attempt to record using an invalid device name
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_recording_invalid.wav");
        let file_path_str = file_path.to_str().unwrap();

        let result = record_audio("InvalidDeviceName", 1, file_path_str);
        assert!(result.is_err());

        // Ensure the file was not created
        assert!(!file_path.exists());

        // Clean up
        temp_dir.close().expect("Failed to delete temp dir");
    }
}
