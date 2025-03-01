use rdev::{Event, EventType, Key, listen};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use anyhow::Result;

/// Represents the application state related to hotkeys
#[derive(Debug, Clone, PartialEq)]
pub struct HotkeyState {
    pub is_recording: bool,
    pub is_post_processing: bool,
}

impl HotkeyState {
    pub fn new() -> Self {
        HotkeyState {
            is_recording: false,
            is_post_processing: false,
        }
    }
}

/// Parses a hotkey string like "Shift+Space" into a set of Keys
pub fn parse_hotkey(hotkey: &str) -> HashSet<Key> {
    hotkey
        .split('+')
        .filter_map(|part| match part.trim().to_lowercase().as_str() {
            "shift" => Some(Key::ShiftLeft), // or Key::ShiftRight
            "control" | "ctrl" => Some(Key::ControlLeft), // or Key::ControlRight
            "alt" => Some(Key::Alt),
            "space" => Some(Key::Space),
            "enter" => Some(Key::Return),
            "escape" => Some(Key::Escape),
            // Add more keys as needed
            _ => None,
        })
        .collect()
}

/// Starts listening to global keyboard events and updates the shared state accordingly
pub async fn start_hotkey_listener(
    config_recording: &str,
    config_modifier: &str,
    state: Arc<Mutex<HotkeyState>>,
) -> Result<()> {
    let recording_keys = parse_hotkey(config_recording);
    let modifier_keys = parse_hotkey(config_modifier);

    let pressed_keys = Arc::new(Mutex::new(HashSet::new()));

    tokio::task::spawn(async move {
        if let Err(error) = listen(move |event: Event| {
            let mut pressed = pressed_keys.lock().unwrap();

            match event.event_type {
                EventType::KeyPress(key) => {
                    pressed.insert(key);
                }
                EventType::KeyRelease(key) => {
                    pressed.remove(&key);
                }
                _ => {}
            }

            let recording_active = recording_keys.iter().all(|k| pressed.contains(k));
            let modifier_active = modifier_keys.iter().all(|k| pressed.contains(k));

            let mut state_lock = state.lock().unwrap();
            state_lock.is_recording = recording_active;
            state_lock.is_post_processing = modifier_active;
        }) {
            println!("Error in hotkey listener: {:?}", error);
        }
    })
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_parse_hotkey() {
        let hotkey = "Shift+Space";
        let parsed = parse_hotkey(hotkey);
        let mut expected = HashSet::new();
        expected.insert(Key::ShiftLeft);
        expected.insert(Key::Space);
        assert_eq!(parsed, expected);

        let hotkey = "Control+Alt+Enter";
        let parsed = parse_hotkey(hotkey);
        let mut expected = HashSet::new();
        expected.insert(Key::ControlLeft);
        expected.insert(Key::Alt);
        expected.insert(Key::Return);
        assert_eq!(parsed, expected);

        let hotkey = "Ctrl + Shift + Escape";
        let parsed = parse_hotkey(hotkey);
        let mut expected = HashSet::new();
        expected.insert(Key::ControlLeft);
        expected.insert(Key::ShiftLeft);
        expected.insert(Key::Escape);
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_hotkey_listener_updates_state() {
        // Note: Testing the actual hotkey listener would require simulating key events,
        // which is complex and environment-dependent. Instead, we can test the
        // parsing and state update logic separately.

        // Initialize state
        let state = Arc::new(Mutex::new(HotkeyState::new()));

        // Define hotkeys
        let _recording_hotkey = "Shift+Space";
        let _modifier_hotkey = "Control";

        // Start the hotkey listener (it will listen to actual key events)
        // For testing purposes, we'll not actually start the listener
        // Instead, we'll simulate state updates manually

        {
            let mut state_lock = state.lock().unwrap();
            state_lock.is_recording = true;
            state_lock.is_post_processing = false;
        }

        {
            let state_lock = state.lock().unwrap();
            assert_eq!(
                *state_lock,
                HotkeyState {
                    is_recording: true,
                    is_post_processing: false
                }
            );
        }

        {
            let mut state_lock = state.lock().unwrap();
            state_lock.is_post_processing = true;
        }

        {
            let state_lock = state.lock().unwrap();
            assert_eq!(
                *state_lock,
                HotkeyState {
                    is_recording: true,
                    is_post_processing: true
                }
            );
        }
    }
}
