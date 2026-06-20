#![cfg(feature = "llm")]

use std::sync::Arc;

/// Embedded GGUF model bytes for Gemma 3 1B-it Q4_K_M quantization
pub static EMBEDDED_GGUF: &[u8] = include_bytes!("../../assets/models/gemma-3-1b-it-Q4_K_M.gguf");

/// Mock MistralRs type for now (real integration will use actual mistralrs)
pub struct MistralRs;

impl std::fmt::Debug for MistralRs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MistralRs").finish()
    }
}

/// Load the embedded Gemma 3 1B-it model from cache or temp directory.
///
/// # Returns
/// - `Ok(Arc<MistralRs>)` on success
/// - `Err(String)` with a descriptive error message on failure
pub async fn load_model() -> Result<Arc<MistralRs>, String> {
    // Construct the stable cache path: {temp_dir}/pikchrmirror/gemma-3-1b-it-Q4_K_M.gguf
    let cache_dir = std::env::temp_dir().join("pikchrmirror");
    let model_path = cache_dir.join("gemma-3-1b-it-Q4_K_M.gguf");

    // Create parent directory if it doesn't exist
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
    }

    // Write the embedded GGUF bytes to the cache path if the file doesn't exist
    if !model_path.exists() {
        std::fs::write(&model_path, EMBEDDED_GGUF)
            .map_err(|e| format!("Failed to write model cache file: {}", e))?;
    }

    // For now, return a mock model
    // TODO: Integrate real mistralrs once API is stable
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    Ok(Arc::new(MistralRs))
}

/// Get the system prompt for the pikchr assistant.
pub fn system_prompt(current_src: &str) -> String {
    format!(
        r#"You are a pikchr diagram assistant. Pikchr is a PIC-like diagram language.

Key pikchr syntax:
- box "label" — draw a box
- circle "label" — draw a circle
- arrow — draw an arrow (follows previous object)
- line — draw a line
- text "label" — add text
- fit — fit to content
- rad 10px — border radius
- right, down, left, up — directions
- 200% — distance/size modifier

Current diagram source:
{}

Generate a pikchr code block for the user's request. Reply with ONLY a single ```pikchr code block containing the complete diagram source."#,
        current_src
    )
}

/// Generate a response from the model.
/// For now this is a mock implementation.
pub async fn generate(
    _model: Arc<MistralRs>,
    _messages: Vec<(String, String)>,
    _system_prompt: String,
) -> Result<String, String> {
    // Simulate thinking time
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    // Return a mock pikchr diagram
    Ok(
        r#"box "Start"
arrow
box "Process"
arrow
box "End"

```pikchr
box "Start"
arrow
box "Process"
arrow
box "End"
```

This is a simple flowchart with three boxes connected by arrows."#
            .to_string(),
    )
}

/// Extract pikchr code from the model's response.
pub fn extract_pikchr_block(reply: &str) -> Option<String> {
    // Find the first ```pikchr block
    if let Some(start_idx) = reply.find("```pikchr") {
        let start = start_idx + 9; // length of "```pikchr"
        if let Some(end_idx) = reply[start..].find("```") {
            let code = reply[start..start + end_idx].trim().to_string();
            if !code.is_empty() {
                return Some(code);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_pikchr_block_valid() {
        let reply = "Here's your diagram:\n```pikchr\nbox \"A\"\n```\nDone!";
        let result = extract_pikchr_block(reply);
        assert_eq!(result, Some("box \"A\"".to_string()));
    }

    #[test]
    fn test_extract_pikchr_block_missing() {
        let reply = "No pikchr here";
        let result = extract_pikchr_block(reply);
        assert_eq!(result, None);
    }

    #[test]
    fn test_system_prompt_includes_context() {
        let prompt = system_prompt("box \"test\"");
        assert!(prompt.contains("box \"test\""));
        assert!(prompt.contains("pikchr"));
    }
}
