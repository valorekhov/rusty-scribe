// src/api.rs

use anyhow::{Result, Context};
use reqwest::blocking::{Client, multipart};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use log::{info, error};

#[derive(Deserialize, Debug, PartialEq)]
pub struct WhisperResponse {
    pub text: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct LLMChoice {
    pub text: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct LLMResponse {
    pub choices: Vec<LLMChoice>,
}

/// Determines whether the local Whisper endpoint is available
pub fn is_local_endpoint_available(url: &str) -> bool {
    let client = Client::new();
    match client.get(url).send() {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Sends the audio file to the specified Whisper endpoint and returns the transcription
pub fn transcribe_audio(
    whisper_url: &str,
    api_key: &str,
    audio_path: &str,
) -> Result<String> {
    let client = Client::new();

    let form = multipart::Form::new()
        .file("file", audio_path)
        .with_context(|| format!("Failed to attach audio file at {}", audio_path))?
        .text("model", "whisper-1");

    let response = client
        .post(whisper_url)
        .multipart(form)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .send()
        .context("Failed to send request to Whisper endpoint")?;

    if response.status().is_success() {
        let whisper_resp: WhisperResponse = response.json()
            .context("Failed to parse Whisper response")?;
        Ok(whisper_resp.text)
    } else {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        Err(anyhow::anyhow!("Whisper API error {}: {}", status, text))
    }
}

/// Sends the transcription to the LLM endpoint for post-processing
pub fn post_process_text(
    llm_url: &str,
    api_key: &str,
    prompt: &str,
    text: &str,
) -> Result<String> {
    let client = Client::new();

    let payload = serde_json::json!({
        "prompt": format!("{} {}", prompt, text),
        "max_tokens": 150,
        "temperature": 0.7,
    });

    let response = client
        .post(llm_url)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()
        .context("Failed to send request to LLM endpoint")?;

    if response.status().is_success() {
        let llm_resp: LLMResponse = response.json()
            .context("Failed to parse LLM response")?;
        if let Some(choice) = llm_resp.choices.into_iter().next() {
            Ok(choice.text.trim().to_string())
        } else {
            Err(anyhow::anyhow!("No choices found in LLM response"))
        }
    } else {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        Err(anyhow::anyhow!("LLM API error {}: {}", status, text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_is_local_endpoint_available_success() {
        let _m = mock("GET", "/health")
            .with_status(200)
            .create();

        let url = &format!("{}/health", &mockito::server_url());
        assert!(is_local_endpoint_available(url));
    }

    #[test]
    fn test_is_local_endpoint_available_failure() {
        let _m = mock("GET", "/health")
            .with_status(500)
            .create();

        let url = &format!("{}/health", &mockito::server_url());
        assert!(!is_local_endpoint_available(url));
    }

    #[test]
    fn test_transcribe_audio_success() {
        let _m = mock("POST", "/transcribe")
            .match_header("authorization", "Bearer test_api_key")
            .match_multipart(Matcher::AllOf(vec![
                Matcher::Exact("model".to_string()),
                Matcher::Exact("whisper-1".to_string()),
                Matcher::Regex("file".to_string(), ".*".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"text": "Transcribed text."}"#)
            .create();

        // Create a temporary audio file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp_file, "dummy audio data").expect("Failed to write to temp file");
        let audio_path = temp_file.path().to_str().unwrap();

        let whisper_url = &format!("{}/transcribe", &mockito::server_url());
        let api_key = "test_api_key";

        let transcription = transcribe_audio(whisper_url, api_key, audio_path).expect("Transcription failed");
        assert_eq!(transcription, "Transcribed text.");
    }

    #[test]
    fn test_transcribe_audio_failure() {
        let _m = mock("POST", "/transcribe")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "Bad Request"}"#)
            .create();

        // Create a temporary audio file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp_file, "dummy audio data").expect("Failed to write to temp file");
        let audio_path = temp_file.path().to_str().unwrap();

        let whisper_url = &format!("{}/transcribe", &mockito::server_url());
        let api_key = "test_api_key";

        let result = transcribe_audio(whisper_url, api_key, audio_path);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Whisper API error 400 Bad Request"
        );
    }

    #[test]
    fn test_post_process_text_success() {
        let _m = mock("POST", "/llm")
            .match_header("authorization", "Bearer test_api_key")
            .match_header("content-type", "application/json")
            .match_body(Matcher::Json(json!({
                "prompt": "Please clean up and format the following text: Transcribed text.",
                "max_tokens": 150,
                "temperature": 0.7
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": [
                    { "text": "Cleaned up and formatted text." }
                ]
            }"#)
            .create();

        let llm_url = &format!("{}/llm", &mockito::server_url());
        let api_key = "test_api_key";
        let prompt = "Please clean up and format the following text:";
        let text = "Transcribed text.";

        let processed_text = post_process_text(llm_url, api_key, prompt, text).expect("Post-processing failed");
        assert_eq!(processed_text, "Cleaned up and formatted text.");
    }

    #[test]
    fn test_post_process_text_no_choices() {
        let _m = mock("POST", "/llm")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": []
            }"#)
            .create();

        let llm_url = &format!("{}/llm", &mockito::server_url());
        let api_key = "test_api_key";
        let prompt = "Please clean up and format the following text:";
        let text = "Transcribed text.";

        let result = post_process_text(llm_url, api_key, prompt, text);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No choices found in LLM response"
        );
    }

    #[test]
    fn test_post_process_text_failure() {
        let _m = mock("POST", "/llm")
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "Internal Server Error"}"#)
            .create();

        let llm_url = &format!("{}/llm", &mockito::server_url());
        let api_key = "test_api_key";
        let prompt = "Please clean up and format the following text:";
        let text = "Transcribed text.";

        let result = post_process_text(llm_url, api_key, prompt, text);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "LLM API error 500 Internal Server Error"
        );
    }
}
