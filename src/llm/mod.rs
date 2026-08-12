#![cfg(feature = "llm")]

use std::sync::Arc;

use mistralrs::{GgufModelBuilder, Model, RequestBuilder, TextMessageRole};

const MODEL_FILE: &str = "Qwen3-0.6B-Q4_K_M.gguf";

/// Embedded GGUF model bytes for Qwen3 0.6B Q4_K_M quantization
pub static EMBEDDED_GGUF: &[u8] = include_bytes!("../../assets/models/Qwen3-0.6B-Q4_K_M.gguf");

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
    // Spawn on a separate task so panics inside the generation are caught as JoinError
    // rather than silently dropped by iced's Task machinery (which would leave `generating`
    // stuck at true forever).
    let handle = tokio::spawn(async move {
        // Use RequestBuilder instead of TextMessages so we can cap max_len.
        // Without a cap, Qwen3 on CPU can generate for minutes.
        // Disable thinking: Qwen3 defaults thinking=true, adding a large <think>... preamble.
        let mut req = RequestBuilder::new()
            .enable_thinking(false)
            .set_sampler_max_len(150)
            .add_message(TextMessageRole::System, system_prompt);
        for (role, text) in messages {
            let r = if role == "assistant" {
                TextMessageRole::Assistant
            } else {
                TextMessageRole::User
            };
            req = req.add_message(r, text);
        }
        model
            .0
            .send_chat_request(req)
            .await
            .map_err(|e| format!("{e}"))
            .and_then(|response| {
                response
                    .choices
                    .first()
                    .and_then(|c| {
                        c.message
                            .content
                            .clone()
                            .or_else(|| c.message.reasoning_content.clone())
                    })
                    .ok_or_else(|| "empty response from model".to_string())
            })
    });

    match tokio::time::timeout(std::time::Duration::from_secs(180), handle).await {
        Ok(Ok(result)) => result,
        Ok(Err(join_err)) => {
            if join_err.is_panic() {
                Err("Model panicked during generation — check logs".to_string())
            } else {
                Err("Generation task was cancelled".to_string())
            }
        }
        Err(_) => Err("Generation timed out after 180 s".to_string()),
    }
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
