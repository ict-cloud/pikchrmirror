#![cfg(feature = "llm")]

use std::path::PathBuf;
use std::sync::Arc;

use mistralrs::{
    DeviceMapSetting, Model, ModelDType, RequestBuilder, TextMessageRole, TextModelBuilder,
};

/// Where build.rs downloaded the model; empty when nothing was bundled.
const BUILT_MODEL_DIR: &str = env!("PIKCHRMIRROR_MODEL_DIR");

/// Maximum number of tokens generated per reply. A complete diagram needs far
/// more than the few dozen tokens a chat answer does.
const MAX_REPLY_TOKENS: usize = 512;

pub struct MistralRs(Model);

impl std::fmt::Debug for MistralRs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MistralRs").finish()
    }
}

/// Find the directory holding the model files: `PIKCHR_MODEL_DIR` (runtime
/// override), else the build-time download location.
fn locate_model_dir(env_dir: Option<PathBuf>, built_dir: &str) -> Result<PathBuf, String> {
    let dir = env_dir
        .or_else(|| (!built_dir.is_empty()).then(|| PathBuf::from(built_dir)))
        .ok_or(
            "no model is bundled in this build. Rebuild with PIKCHR_MODEL=<id> \
                (see models.toml) or set PIKCHR_MODEL_DIR to a model directory",
        )?;
    if dir.join("config.json").is_file() {
        Ok(dir)
    } else {
        Err(format!(
            "model files not found in {}. Rebuild to download them or set PIKCHR_MODEL_DIR",
            dir.display()
        ))
    }
}

pub async fn load_model() -> Result<Arc<MistralRs>, String> {
    let dir = locate_model_dir(
        std::env::var_os("PIKCHR_MODEL_DIR").map(PathBuf::from),
        BUILT_MODEL_DIR,
    )?;

    // f32: candle's CPU matmul has no BF16 kernel, and mistralrs would otherwise
    // pick BF16 on macOS. No ISQ: at 0.5B f32 is ~2 GB and 4-bit costs quality.
    let model = TextModelBuilder::new(dir.to_string_lossy().into_owned())
        .with_dtype(ModelDType::F32)
        .with_force_cpu()
        // Skip mistralrs' memory-based auto device mapping: on some hosts it misreads
        // available CPU RAM as 0MB and refuses to load any model. We only ever run on
        // a single forced CPU device, so just place all layers there directly.
        .with_device_mapping(DeviceMapSetting::dummy())
        .build()
        .await
        .map_err(|e| format!("Failed to load model from {}: {e}", dir.display()))?;

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
    // RequestBuilder instead of TextMessages so we can cap max_len: without a cap,
    // generation on CPU can run for minutes. Thinking is off so reasoning-capable
    // models skip their <think>... preamble.
    let mut req = RequestBuilder::new()
        .enable_thinking(false)
        .set_sampler_max_len(MAX_REPLY_TOKENS)
        .add_message(TextMessageRole::System, system_prompt);
    for (role, text) in messages {
        let r = if role == "assistant" {
            TextMessageRole::Assistant
        } else {
            TextMessageRole::User
        };
        req = req.add_message(r, text);
    }
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(180),
        model.0.send_chat_request(req),
    )
    .await
    .map_err(|_| "Generation timed out after 180 s".to_string())?
    .map_err(|e| e.to_string())?;
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
    fn test_locate_model_dir() {
        let dir = std::env::temp_dir().join(format!("pikchr-locate-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.json"), "{}").unwrap();
        let built = dir.to_string_lossy();
        assert_eq!(
            locate_model_dir(Some(dir.clone()), "/nonexistent"),
            Ok(dir.clone())
        );
        assert_eq!(locate_model_dir(None, &built), Ok(dir.clone()));
        assert!(locate_model_dir(None, "")
            .unwrap_err()
            .contains("PIKCHR_MODEL"));
        assert!(locate_model_dir(None, "/nonexistent")
            .unwrap_err()
            .contains("/nonexistent"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// End-to-end check of the real load + generate path against a model directory.
    /// Ignored by default (needs weights). Run it with, for example:
    /// `PIKCHR_MODEL_DIR=/path/to/model cargo test -p PikchrMirror --features llm -- --ignored --nocapture`
    #[tokio::test]
    #[ignore = "needs a model directory in PIKCHR_MODEL_DIR"]
    async fn smoke_load_and_generate() {
        let model = load_model().await.expect("model loads");
        let reply = generate(
            model,
            vec![("user".to_string(), "draw me a box".to_string())],
            system_prompt("box"),
        )
        .await
        .expect("model replies");
        println!("reply: {reply}");
        assert!(!reply.is_empty());
    }

    #[test]
    fn test_system_prompt_includes_context() {
        let prompt = system_prompt("box \"test\"");
        assert!(prompt.contains("box \"test\""));
        assert!(prompt.contains("pikchr"));
    }
}
