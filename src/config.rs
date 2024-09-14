// src/config.rs

use serde::Deserialize;
use std::fs;
use anyhow::{Result, Context};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    pub endpoints: Endpoints,
    pub hotkeys: Hotkeys,
    pub audio: AudioSettings,
    pub llm: LLMSettings,
    pub api_keys: ApiKeys,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Endpoints {
    pub local_whisper: String,
    pub hosted_whisper: String,
    pub llm_endpoint: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Hotkeys {
    pub recording: String,
    pub post_processing_modifier: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct AudioSettings {
    pub recording_device: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct LLMSettings {
    pub post_processing_prompt: String,
    pub always_post_process: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ApiKeys {
    pub openai: String,
}

pub fn load_config() -> Result<Config> {
    let config_content = fs::read_to_string("config.toml")
        .context("Unable to read config.toml. Ensure the file exists in the project root.")?;
    let config: Config = toml::from_str(&config_content)
        .context("Error parsing config.toml. Please check the file's syntax.")?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_load_config_success() {
        // Create a temporary config file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_content = r#"
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
            openai = "test_openai_api_key"
        "#;
        write!(temp_file, "{}", config_content).expect("Failed to write to temp file");

        // Temporarily rename the temp file to 'config.toml'
        let temp_path = temp_file.path().to_path_buf();
        let original_config = "config.toml";
        let backup_path = "config_backup.toml";

        // Backup existing config.toml if it exists
        if std::path::Path::new(original_config).exists() {
            fs::rename(original_config, backup_path).expect("Failed to backup original config.toml");
        }

        // Copy temp file to config.toml
        fs::copy(&temp_path, original_config).expect("Failed to copy temp config to config.toml");

        // Load config
        let loaded_config = load_config().expect("Failed to load config");

        // Define expected config
        let expected_config = Config {
            endpoints: Endpoints {
                local_whisper: "http://localhost:5000/transcribe".to_string(),
                hosted_whisper: "https://api.openai.com/v1/audio/transcriptions".to_string(),
                llm_endpoint: "https://api.openai.com/v1/engines/davinci/completions".to_string(),
            },
            hotkeys: Hotkeys {
                recording: "Shift+Space".to_string(),
                post_processing_modifier: "Control".to_string(),
            },
            audio: AudioSettings {
                recording_device: "default".to_string(),
            },
            llm: LLMSettings {
                post_processing_prompt: "Please clean up and format the following text:".to_string(),
                always_post_process: false,
            },
            api_keys: ApiKeys {
                openai: "test_openai_api_key".to_string(),
            },
        };

        assert_eq!(loaded_config, expected_config);

        // Restore original config.toml if it was backed up
        if std::path::Path::new(backup_path).exists() {
            fs::rename(backup_path, original_config).expect("Failed to restore original config.toml");
        } else {
            fs::remove_file(original_config).expect("Failed to remove temp config.toml");
        }
    }

    #[test]
    fn test_load_config_failure_missing_file() {
        // Temporarily rename config.toml to simulate missing file
        let original_config = "config.toml";
        let backup_path = "config_backup.toml";

        if std::path::Path::new(original_config).exists() {
            fs::rename(original_config, backup_path).expect("Failed to backup original config.toml");
        }

        // Attempt to load config
        let result = load_config();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unable to read config.toml. Ensure the file exists in the project root."
        );

        // Restore original config.toml if it was backed up
        if std::path::Path::new(backup_path).exists() {
            fs::rename(backup_path, original_config).expect("Failed to restore original config.toml");
        }
    }

    #[test]
    fn test_load_config_failure_invalid_toml() {
        // Create a temporary invalid config file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let invalid_config_content = r#"
            [endpoints]
            local_whisper = "http://localhost:5000/transcribe
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
            openai = "test_openai_api_key"
        "#; // Note the missing closing quote for local_whisper

        write!(temp_file, "{}", invalid_config_content).expect("Failed to write to temp file");

        // Temporarily rename the temp file to 'config.toml'
        let temp_path = temp_file.path().to_path_buf();
        let original_config = "config.toml";
        let backup_path = "config_backup.toml";

        // Backup existing config.toml if it exists
        if std::path::Path::new(original_config).exists() {
            fs::rename(original_config, backup_path).expect("Failed to backup original config.toml");
        }

        // Copy temp file to config.toml
        fs::copy(&temp_path, original_config).expect("Failed to copy temp config to config.toml");

        // Attempt to load config
        let result = load_config();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Error parsing config.toml"));

        // Restore original config.toml if it was backed up
        if std::path::Path::new(backup_path).exists() {
            fs::rename(backup_path, original_config).expect("Failed to restore original config.toml");
        } else {
            fs::remove_file(original_config).expect("Failed to remove temp config.toml");
        }
    }
}
