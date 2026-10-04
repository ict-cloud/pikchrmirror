#![cfg(feature = "llm")]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mistralrs::{
    DeviceMapSetting, IsqBits, Model, RequestBuilder, TextMessageRole, TextModelBuilder,
};

/// Id of the model selected at build time (`models.toml` / `PIKCHR_MODEL`);
/// empty when the build was made with `PIKCHR_MODEL=none`.
const MODEL_ID: &str = env!("PIKCHRMIRROR_MODEL_ID");
/// Where build.rs downloaded that model; empty when nothing was bundled.
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

/// Find the directory holding the model files.
///
/// Order: `PIKCHR_MODEL_DIR` (runtime override), the build-time download
/// location, then `models/<id>` next to the executable (for relocated installs).
fn locate_model_dir(
    env_dir: Option<PathBuf>,
    built_dir: &str,
    exe_dir: Option<PathBuf>,
    id: &str,
) -> Result<PathBuf, String> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    candidates.extend(env_dir);
    if !built_dir.is_empty() {
        candidates.push(PathBuf::from(built_dir));
    }
    if let (Some(exe_dir), false) = (exe_dir, id.is_empty()) {
        candidates.push(exe_dir.join("models").join(id));
    }

    if let Some(found) = candidates.iter().find(|d| d.join("config.json").is_file()) {
        return Ok(found.clone());
    }
    if candidates.is_empty() {
        return Err(
            "no model is bundled in this build. Rebuild with PIKCHR_MODEL=<id> \
             (see models.toml) or set PIKCHR_MODEL_DIR to a model directory"
                .to_string(),
        );
    }
    let tried: Vec<String> = candidates.iter().map(|d| d.display().to_string()).collect();
    Err(format!(
        "model files not found (looked in: {}). Rebuild to download them or set PIKCHR_MODEL_DIR",
        tried.join(", ")
    ))
}

/// Whether `dir` holds a chat template (`chat_template.jinja`, `chat_template.json`
/// or a `chat_template` entry in `tokenizer_config.json`). Without one every chat
/// request is rejected, so check up front for a clear error.
fn has_chat_template(dir: &Path) -> bool {
    dir.join("chat_template.jinja").is_file()
        || dir.join("chat_template.json").is_file()
        || std::fs::read_to_string(dir.join("tokenizer_config.json"))
            .is_ok_and(|s| s.contains("\"chat_template\""))
}

pub async fn load_model() -> Result<Arc<MistralRs>, String> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    let dir = locate_model_dir(
        std::env::var_os("PIKCHR_MODEL_DIR").map(PathBuf::from),
        BUILT_MODEL_DIR,
        exe_dir,
        MODEL_ID,
    )?;
    if !has_chat_template(&dir) {
        return Err(format!(
            "no chat template in {} (expected chat_template.jinja, chat_template.json or a \
             `chat_template` in tokenizer_config.json). Rebuild with --features llm to \
             re-download the model files",
            dir.display()
        ));
    }

    // Weights ship as bf16 safetensors; quantize to 4 bit on load to keep RAM
    // and CPU inference time reasonable.
    let model = TextModelBuilder::new(dir.to_string_lossy().into_owned())
        .with_auto_isq(IsqBits::Four)
        .with_force_cpu()
        // Skip mistralrs' memory-based auto device mapping: on some hosts it misreads
        // available CPU RAM as 0MB and refuses to load any model. We only ever run on
        // a single forced CPU device, so just place all layers there directly.
        .with_device_mapping(DeviceMapSetting::dummy())
        .build()
        .await
        .map_err(|e| format!("Failed to load model `{MODEL_ID}`: {e}"))?;

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
        // Without a cap, generation on CPU can run for minutes.
        // Disable thinking so reasoning-capable models skip their <think>... preamble.
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
    fn test_locate_model_dir_prefers_env_then_built_then_exe() {
        let root = std::env::temp_dir().join(format!("pikchr-locate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let make = |name: &str| {
            let dir = root.join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("config.json"), "{}").unwrap();
            dir
        };
        let (env_dir, built, exe) = (make("env"), make("built"), root.join("exe"));
        std::fs::create_dir_all(exe.join("models")).unwrap();
        let bundled = exe.join("models").join("granite");
        std::fs::create_dir_all(&bundled).unwrap();
        std::fs::write(bundled.join("config.json"), "{}").unwrap();
        let built_str = built.to_string_lossy();

        let found = locate_model_dir(
            Some(env_dir.clone()),
            &built_str,
            Some(exe.clone()),
            "granite",
        );
        assert_eq!(found, Ok(env_dir));
        let found = locate_model_dir(None, &built_str, Some(exe.clone()), "granite");
        assert_eq!(found, Ok(built));
        let found = locate_model_dir(None, "/nonexistent", Some(exe), "granite");
        assert_eq!(found, Ok(bundled));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_locate_model_dir_errors_are_actionable() {
        let none = locate_model_dir(None, "", None, "").unwrap_err();
        assert!(none.contains("PIKCHR_MODEL"), "{none}");
        let missing = locate_model_dir(None, "/nonexistent/model", None, "granite").unwrap_err();
        assert!(missing.contains("/nonexistent/model"), "{missing}");
    }

    #[test]
    fn test_has_chat_template() {
        let dir = std::env::temp_dir().join(format!("pikchr-tmpl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(!has_chat_template(&dir));
        std::fs::write(
            dir.join("tokenizer_config.json"),
            r#"{"model_max_length": 8}"#,
        )
        .unwrap();
        assert!(!has_chat_template(&dir));
        std::fs::write(
            dir.join("tokenizer_config.json"),
            r#"{"chat_template": "x"}"#,
        )
        .unwrap();
        assert!(has_chat_template(&dir));
        std::fs::remove_file(dir.join("tokenizer_config.json")).unwrap();
        std::fs::write(dir.join("chat_template.jinja"), "x").unwrap();
        assert!(has_chat_template(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_system_prompt_includes_context() {
        let prompt = system_prompt("box \"test\"");
        assert!(prompt.contains("box \"test\""));
        assert!(prompt.contains("pikchr"));
    }
}
