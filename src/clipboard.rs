use anyhow::Result;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use log::info;

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize clipboard context: {}", e))?;
    ctx.set_contents(text.to_owned())
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard contents: {}", e))?;
    info!("Text copied to clipboard.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipboard::ClipboardProvider;
    use clipboard::ClipboardContext;
    use anyhow::Result;

    #[test]
    fn test_copy_to_clipboard_success() -> Result<()> {
        let test_text = "Test clipboard text.";
        copy_to_clipboard(test_text)?;

        // Retrieve the text from the clipboard to verify
        let mut ctx: ClipboardContext = ClipboardProvider::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize clipboard context: {}", e))?;
        let clipboard_content = ctx.get_contents()
            .map_err(|e| anyhow::anyhow!("Failed to get clipboard contents: {}", e))?;

        assert_eq!(clipboard_content, test_text);
        Ok(())
    }

    #[test]
    fn test_copy_to_clipboard_failure() {
        // This test is environment-dependent and might not be feasible.
        // For example, if the clipboard is inaccessible, it would fail.
        // Instead, we can mock the ClipboardProvider, but the `clipboard` crate
        // doesn't support mocking out of the box.

        // Therefore, we acknowledge that testing failure cases here is non-trivial
        // and would require refactoring the code to allow dependency injection.
        // For simplicity, we'll skip this test.

        // Example:
        // let result = copy_to_clipboard("This should fail");
        // assert!(result.is_err());
    }
}
