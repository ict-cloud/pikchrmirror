#![cfg(feature = "llm")]

use std::sync::Arc;

use mistralrs::{GgufModelBuilder, Model, TextMessageRole, TextMessages};

const MODEL_FILE: &str = "gemma-3-1b-it-Q4_K_M.gguf";

/// Embedded GGUF model bytes for Gemma 3 1B-it Q4_K_M quantization
pub static EMBEDDED_GGUF: &[u8] = include_bytes!("../../assets/models/gemma-3-1b-it-Q4_K_M.gguf");

pub struct MistralRs(Model);

impl std::fmt::Debug for MistralRs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MistralRs").finish()
    }
}

pub async fn load_model() -> Result<Arc<MistralRs>, String> {
    let cache_dir = std::env::temp_dir().join("pikchrmirror");
    let model_path = cache_dir.join(MODEL_FILE);
    std::fs::create_dir_all(&cache_dir).map_err(|e| format!("create cache dir: {e}"))?;

    // Refresh if the cached file size differs from the embedded bytes (e.g. stale 1-byte placeholder).
    let needs_write = std::fs::metadata(&model_path)
        .map(|m| m.len() as usize != EMBEDDED_GGUF.len())
        .unwrap_or(true);
    if needs_write {
        std::fs::write(&model_path, EMBEDDED_GGUF).map_err(|e| format!("write model: {e}"))?;
    }

    let model = GgufModelBuilder::new(
        cache_dir.to_string_lossy().into_owned(),
        vec![MODEL_FILE.to_string()],
    )
    .with_force_cpu()
    .build()
    .await
    .map_err(|e| format!("Failed to load model: {e}"))?;

    Ok(Arc::new(MistralRs(model)))
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

pub async fn generate(
    model: Arc<MistralRs>,
    messages: Vec<(String, String)>,
    system_prompt: String,
) -> Result<String, String> {
    let mut tm = TextMessages::new().add_message(TextMessageRole::System, system_prompt);
    for (role, text) in messages {
        let r = if role == "assistant" {
            TextMessageRole::Assistant
        } else {
            TextMessageRole::User
        };
        tm = tm.add_message(r, text);
    }
    let response = model
        .0
        .send_chat_request(tm)
        .await
        .map_err(|e| format!("{e}"))?;
    response
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .ok_or_else(|| "empty response from model".to_string())
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
