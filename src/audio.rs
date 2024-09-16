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
pub fn record_audio(device_name: &str, duration_secs: u64, tx: mpsc::Sender<i16>) -> Result<()> {
    let device = get_device_from_name( device_name)?;

    info!("Using audio device: {}", device.name()?);

    let config = device.default_input_config().context("Failed to get default input config")?;

    let sample_format = config.sample_format();
    let config: cpal::StreamConfig = config.into();

    // Build and run the stream
    let stream = match sample_format {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, tx.clone())?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, tx.clone())?,
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config, tx.clone())?,
        _ => return Err(anyhow::anyhow!("Unsupported sample format")),
    };

    stream.play().context("Failed to start audio stream")?;

    info!("Recording audio for {} seconds...", duration_secs);

    std::thread::sleep(Duration::from_secs(duration_secs));

    drop(stream);

    info!("Audio recording completed");
    Ok(())
}

pub fn get_device_from_name(device_name: &str) -> Result<cpal::Device> {
    let host = cpal::default_host();
    if device_name.to_lowercase() == "default" {
        host.default_input_device().context("No default input device available")
    } else {
        host.input_devices()
            .context("Failed to get input devices")?
            .find(|d| d.name().map(|n| n == device_name).unwrap_or(false))
            .context("Specified recording device not found")
    }
}

pub fn save_audio_to_wav(rx: mpsc::Receiver<i16>, file_path: &str, config: &cpal::StreamConfig) -> Result<()> {
    // Setup WAV writer
    let spec = WavSpec {
        channels: config.channels,
        sample_rate: config.sample_rate.0,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(file_path, spec)
        .with_context(|| format!("Failed to create WAV file at {}", file_path))?;

    while let Ok(sample) = rx.recv() {
        writer.write_sample(sample)
            .context("Failed to write audio sample to WAV")?;
    }

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
        // Record a short audio snippet and ensure data is sent to the buffer.
        // Note: This test will actually record audio from the default device.
        // It's better to mock the audio input, but for simplicity, we'll perform a real recording.

        let (sender, receiver) = std::sync::mpsc::channel::<i16>();

        // Increase the duration to ensure we get a complete number of samples
        let result = record_audio("default", 2, sender);
        if let Err(e) = &result {
            eprintln!("Error recording audio: {:?}", e);
        }
        assert!(result.is_ok());

        // Check that we received some data
        let received: Vec<i16> = receiver.iter().collect();
        assert!(!received.is_empty());

        // No need for cleanup as we're using in-memory buffer
    }

    #[test]
    fn test_record_audio_invalid_device() {
        let (sender, _) = std::sync::mpsc::channel::<i16>();
        let result = record_audio("InvalidDeviceName", 1, sender);
        assert!(result.is_err());
    }
}
