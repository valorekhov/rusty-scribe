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
