## Table of Contents

1. [Project Overview](#project-overview)
2. [Prerequisites](#prerequisites)
3. [Project Structure](#project-structure)
4. [Configuration Management](#configuration-management)
5. [Handling Hotkeys](#handling-hotkeys)
6. [Audio Recording](#audio-recording)
7. [Interacting with Whisper and LLM Endpoints](#interacting-with-whisper-and-llm-endpoints)
8. [Clipboard Management](#clipboard-management)
9. [Putting It All Together](#putting-it-all-together)
10. [Running the Application](#running-the-application)

---

## Project Overview

The application performs the following tasks:

1. **Configuration Management**: Reads configuration parameters from a file.
2. **Hotkey Handling**: Listens for specific hotkey combinations to start and stop audio recording.
3. **Audio Recording**: Records audio from a selected device while the recording hotkey is pressed.
4. **Data Processing**:
   - Sends the recorded audio to a Whisper endpoint for transcription.
   - Optionally processes the transcription through an LLM for post-processing.
5. **Clipboard Management**: Copies the final text to the clipboard.

## Prerequisites

- **Rust Toolchain**: Ensure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/).
- **FFmpeg**: Some audio processing might require FFmpeg. Install it from [FFmpeg Downloads](https://ffmpeg.org/download.html).
- **API Keys**: If using hosted endpoints like OpenAI, ensure you have the necessary API keys.

## Project Structure

```
whisper_llm_app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── hotkeys.rs
│   ├── audio.rs
│   ├── api.rs
│   └── clipboard.rs
└── config.toml
```

For simplicity, we'll present the code in a single `main.rs` file. However, for better organization and maintainability, consider splitting the code into multiple modules as shown above.

## Configuration Management

We'll use `serde` and `toml` crates to manage configuration. Create a `config.toml` file in the project root:

```toml
# config.toml

[endpoints]
local_whisper = "http://localhost:5000/transcribe"
hosted_whisper = "https://api.openai.com/v1/audio/transcriptions"
llm_endpoint = "https://api.openai.com/v1/engines/davinci/completions"

[hotkeys]
recording = "Shift+Space"
post_processing_modifier = "Control"

[audio]
recording_device = "default"

[llm]
post_processing_prompt = "Please clean up and format the following text:"
always_post_process = false

[api_keys]
openai = "your_openai_api_key_here"
```

## Code Explanation

1. **Configuration Loading**:
    - The `Config` struct mirrors the `config.toml` structure.
    - The `load_config` function reads and parses the `config.toml` file.

2. **Audio Devices Listing**:
    - The `list_audio_devices` function lists all available input audio devices. This can help users configure the correct device.

3. **Hotkey Handling**:
    - Using the `rdev` crate, the application listens for key press and release events.
    - When the recording hotkey is pressed, it sets `is_recording` to `true`.
    - When released, it sets `is_recording` to `false` and proceeds with processing.
    - Similarly, it listens for the post-processing modifier key.

4. **Audio Recording**:
    - **Placeholder Implementation**: For simplicity, the code simulates audio recording by sleeping for 5 seconds. In a real application, you'd use the `cpal` crate to capture audio from the selected device and save it as a WAV file.
    - **TODO**: Implement actual audio recording logic and save the recording to a file (e.g., `recording.wav`).

5. **Interacting with Whisper Endpoint**:
    - Depending on whether a local or hosted Whisper endpoint is used, it selects the appropriate URL.
    - Sends the recorded audio file using a multipart/form-data POST request.
    - Parses the transcription from the response.

6. **Post-Processing with LLM**:
    - If post-processing is enabled (either via modifier key or the `always_post_process` flag), the transcription is sent to the LLM endpoint with the configured prompt.
    - Parses the LLM's response to get the final text.

7. **Clipboard Management**:
    - Uses the `clipboard` crate to copy the final text to the system clipboard.

8. **User Prompts**:
    - If a hosted Whisper endpoint is used, it prompts the user to confirm whether the data contains sensitive information before proceeding.

9. **Helper Function**:
    - `key_matches` is a simplified function to match key events with configured hotkeys. For a more robust solution, consider implementing a comprehensive parser.


**Notes**:

- You'll need to add the `hound` crate for writing WAV files. Add `hound = "3.4.0"` to your `Cargo.toml`.
- Modify the main loop to call `record_audio` instead of simulating.

**Example Integration in Main Loop**:

```rust
// Replace the placeholder recording with actual recording
let audio_path = "recording.wav";
if let Err(e) = record_audio(&config.audio.recording_device, 5, audio_path) {
    println!("Audio recording error: {:?}", e);
    continue;
}
```

## Handling Network Detection for Local Endpoints

To determine if a local Whisper endpoint is available on the same network, you can attempt to send a simple request to the local endpoint and fallback to the hosted endpoint if unavailable.

**Example**:

```rust
fn is_local_endpoint_available(url: &str) -> bool {
    let client = Client::new();
    match client.get(url).send() {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}
```

**Usage**:

```rust
let whisper_url = if is_local_endpoint_available(&config.endpoints.local_whisper) {
    config.endpoints.local_whisper.clone()
} else {
    config.endpoints.hosted_whisper.clone()
};
```

## Improving Hotkey Handling

The current `key_matches` function is simplistic. For a robust solution:

1. **Parse Hotkey Strings**: Break down the hotkey string (e.g., "Shift+Space") into components.
2. **Maintain State of Pressed Keys**: Keep track of currently pressed keys to detect combinations.

Consider using a crate like `hotkey` or enhancing the existing logic.

## Error Handling and Logging

For production-ready applications:

- **Use Logging**: Integrate the `log` and `env_logger` crates to handle logging.
- **Comprehensive Error Handling**: Replace `unwrap` with proper error handling to prevent panics.
- **Graceful Shutdown**: Ensure all threads and resources are cleaned up on exit.

## Running the Application

1. **Configure**:
    - Update `config.toml` with your endpoints and API keys.
    - Ensure the selected audio device exists.

2. **Build**:

    ```bash
    cargo build --release
    ```

3. **Run**:

    ```bash
    cargo run --release
    ```

4. **Usage**:
    - Press the configured recording hotkey (e.g., Shift+Space) to start recording.
    - Release the hotkey to stop recording and process the audio.
    - If post-processing is enabled or the modifier key is pressed, the transcription will be sent to the LLM.
    - The final text is copied to the clipboard.
