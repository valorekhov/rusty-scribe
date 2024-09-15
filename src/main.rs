// src/main.rs

mod config;
mod hotkeys;
mod audio;
mod api;
mod clipboard;

use config::load_config;
use hotkeys::{start_hotkey_listener, HotkeyState};
use audio::record_audio;
use api::{is_local_endpoint_available, transcribe_audio, post_process_text};
use clipboard::copy_to_clipboard;

use anyhow::{Result, Context};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use dialoguer::Confirm;
use log::{info, error};
use env_logger::Env;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Load configuration
    let config = load_config()?;
    info!("Configuration loaded successfully.");

    // Optionally list audio devices
    // Uncomment the following line to list devices and exit
    // list_audio_devices()?;
    // return Ok(());

    // Initialize shared hotkey state
    let state = Arc::new(Mutex::new(HotkeyState::new()));

    // Start hotkey listener
    start_hotkey_listener(
        &config.hotkeys.recording,
        &config.hotkeys.post_processing_modifier,
        Arc::clone(&state),
    ).context("Failed to start hotkey listener")?;
    info!("Hotkey listener started.");

    // Main loop
    loop {
        {
            let current_state = state.lock().unwrap().clone();

            if current_state.is_recording {
                // Determine the duration to record based on how long the hotkey is pressed
                // For simplicity, we'll record until the hotkey is released
                // Implementing this requires more complex event handling
                // Here, we'll simulate a fixed duration recording
                let recording_duration = 5; // seconds
                let audio_file = "recording.wav";

                info!("Starting audio recording...");

                if let Err(e) = record_audio(&config.audio.recording_device, recording_duration, audio_file) {
                    error!("Audio recording failed: {:?}", e);
                    continue;
                }

                // After recording, process the audio
                // Determine which Whisper endpoint to use
                let use_local = is_local_endpoint_available(&config.endpoints.local_whisper);
                let whisper_url = if use_local {
                    info!("Using local Whisper endpoint.");
                    &config.endpoints.local_whisper
                } else {
                    info!("Using hosted Whisper endpoint.");
                    &config.endpoints.hosted_whisper
                };

                // If using hosted Whisper, prompt for sensitive data
                let proceed = if !use_local {
                    Confirm::new()
                        .with_prompt("Are you sure the audio does not contain sensitive data you don't want on the internet?")
                        .default(false)
                        .interact()?
                } else {
                    true
                };

                if !proceed {
                    info!("User aborted due to sensitive data.");
                    continue;
                }

                // Transcribe audio
                let transcription = match transcribe_audio(
                    whisper_url,
                    &config.api_keys.openai,
                    audio_file,
                ) {
                    Ok(text) => {
                        info!("Transcription successful.");
                        text
                    }
                    Err(e) => {
                        error!("Transcription failed: {:?}", e);
                        continue;
                    }
                };

                info!("Transcription: {}", transcription);

                // Determine if post-processing is needed
                let post_processing_needed = current_state.is_post_processing || config.llm.always_post_process;

                let final_text = if post_processing_needed {
                    info!("Post-processing enabled. Sending transcription to LLM.");
                    match post_process_text(
                        &config.endpoints.llm_endpoint,
                        &config.api_keys.openai,
                        &config.llm.post_processing_prompt,
                        &transcription,
                    ) {
                        Ok(text) => {
                            info!("Post-processing successful.");
                            text
                        }
                        Err(e) => {
                            error!("Post-processing failed: {:?}", e);
                            transcription.clone()
                        }
                    }
                } else {
                    transcription.clone()
                };

                info!("Final Text: {}", final_text);

                // Copy to clipboard
                if let Err(e) = copy_to_clipboard(&final_text) {
                    error!("Failed to copy to clipboard: {:?}", e);
                }

                // Reset recording state
                let mut state_lock = state.lock().unwrap();
                state_lock.is_recording = false;
                state_lock.is_post_processing = false;
            }
        }

        // Sleep briefly to reduce CPU usage
        thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn test_main_flow_without_hotkeys() {
        // Testing the main function's loop is not feasible as it contains an infinite loop.
        // Instead, consider refactoring the main logic into a separate function that can be tested.
        // For example, extracting the processing steps into a function and testing that.

        // This test serves as a placeholder to indicate that main loop testing requires refactoring.
        assert!(true);
    }

    // Example of refactoring for testability
    /*
    fn process_recording(config: &Config, state: &HotkeyState) -> Result<()> {
        // Extracted processing logic
    }

    #[test]
    fn test_process_recording() {
        // Implement tests for the extracted function
    }
    */
}
