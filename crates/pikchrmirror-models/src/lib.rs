//! Model registry and build-time model download.
//!
//! The registry (`models.toml` in the workspace root) lists the models the
//! optional `llm` feature of the app can bundle. This crate resolves which one
//! is selected and downloads its files from a Hugging Face compatible endpoint,
//! so the (multi-GB) weights never have to live in the git repository.
//!
//! It is only ever used from the app's `build.rs`, and only when the `llm`
//! feature is enabled, so default builds do not compile it.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, String>;

/// Selection value that disables bundling (used for CI compile checks).
pub const DISABLED: &str = "none";

/// Default Hugging Face endpoint; overridable with `HF_ENDPOINT` for mirrors.
pub const DEFAULT_ENDPOINT: &str = "https://huggingface.co";

const STAMP_FILE: &str = ".pikchrmirror-model.json";

fn default_revision() -> String {
    "main".to_string()
}

fn default_include() -> Vec<String> {
    // `*.jinja` carries `chat_template.jinja`, where current repositories keep the
    // chat template; without it the model cannot be used for chat.
    [
        "*.json",
        "*.jinja",
        "*.safetensors",
        "*.txt",
        "tokenizer.model",
    ]
    .map(String::from)
    .to_vec()
}

fn default_supported() -> bool {
    true
}

/// One entry of the `[models.<id>]` table in `models.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModelSpec {
    /// Registry key, filled in after parsing.
    #[serde(skip)]
    pub id: String,
    pub name: String,
    /// Hugging Face repository, e.g. `ibm-granite/granite-4.0-h-micro`.
    pub repo: String,
    /// Branch, tag or commit. Pin a commit for reproducible builds.
    #[serde(default = "default_revision")]
    pub revision: String,
    /// Top-level file name patterns (`*` wildcard) to download.
    #[serde(default = "default_include")]
    pub include: Vec<String>,
    /// Whether the inference backend can load this model.
    #[serde(default = "default_supported")]
    pub supported: bool,
    /// Why the model is unsupported, shown when it is selected anyway.
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Registry {
    pub selected: String,
    pub models: BTreeMap<String, ModelSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    /// Nothing is downloaded; the app compiles but has no model bundled.
    Disabled,
    Model(ModelSpec),
}

impl Registry {
    pub fn parse(source: &str) -> Result<Self> {
        let mut registry: Registry =
            toml::from_str(source).map_err(|e| format!("invalid model registry: {e}"))?;
        for (id, spec) in registry.models.iter_mut() {
            spec.id = id.clone();
        }
        Ok(registry)
    }

    pub fn load(path: &Path) -> Result<Self> {
        let source = fs::read_to_string(path)
            .map_err(|e| format!("cannot read model registry {}: {e}", path.display()))?;
        Self::parse(&source)
    }

    /// Resolve the model to bundle. `override_id` (the `PIKCHR_MODEL` env var)
    /// wins over the registry's `selected` key; `none` disables bundling.
    pub fn resolve(&self, override_id: Option<&str>) -> Result<Selection> {
        let id = override_id
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(&self.selected);
        if id.eq_ignore_ascii_case(DISABLED) {
            return Ok(Selection::Disabled);
        }
        let spec = self.models.get(id).ok_or_else(|| {
            let known: Vec<&str> = self.models.keys().map(String::as_str).collect();
            format!(
                "unknown model `{id}`; choose one of [{}] or `{DISABLED}`",
                known.join(", ")
            )
        })?;
        if !spec.supported {
            return Err(format!(
                "model `{id}` ({}) cannot be used: {}. Select a supported model or `{DISABLED}`.",
                spec.name,
                spec.reason
                    .as_deref()
                    .unwrap_or("unsupported by the backend"),
            ));
        }
        Ok(Selection::Model(spec.clone()))
    }
}

/// Match `name` against a pattern where `*` matches any run of characters.
pub fn matches_pattern(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == name;
    }
    let (first, last) = (parts[0], parts[parts.len() - 1]);
    if !name.starts_with(first) || name.len() < first.len() + last.len() {
        return false;
    }
    let mut rest = &name[first.len()..name.len() - last.len()];
    if !name.ends_with(last) {
        return false;
    }
    for part in &parts[1..parts.len() - 1] {
        match rest.find(part) {
            Some(i) => rest = &rest[i + part.len()..],
            None => return false,
        }
    }
    true
}

/// A file may be downloaded when it is top-level, safe to write, and matches
/// one of the include patterns.
pub fn is_wanted(name: &str, include: &[String]) -> bool {
    !name.is_empty()
        && !name.contains(['/', '\\'])
        && !name.starts_with('.')
        && include.iter().any(|p| matches_pattern(p, name))
}

#[derive(Debug, Clone, Default)]
pub struct FetchOptions {
    /// Endpoint without trailing slash; falls back to [`DEFAULT_ENDPOINT`].
    pub endpoint: Option<String>,
    /// Access token for gated repositories.
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RevisionInfo {
    #[serde(default)]
    sha: Option<String>,
    #[serde(default)]
    siblings: Vec<Sibling>,
}

#[derive(Debug, Deserialize)]
struct Sibling {
    rfilename: String,
    #[serde(default)]
    size: Option<u64>,
}

/// Written next to the weights once every file has been downloaded, so later
/// builds can skip the network entirely.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Stamp {
    repo: String,
    revision: String,
    include: Vec<String>,
    commit: Option<String>,
    files: Vec<String>,
}

fn encode_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Whether `dir` holds a chat template in any of the places mistralrs reads:
/// `chat_template.jinja`, `chat_template.json`, or the `chat_template` field of
/// `tokenizer_config.json`.
pub fn has_chat_template(dir: &Path) -> bool {
    if dir.join("chat_template.jinja").is_file() || dir.join("chat_template.json").is_file() {
        return true;
    }
    fs::read_to_string(dir.join("tokenizer_config.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("chat_template").cloned())
        .is_some_and(|t| !t.is_null() && t != "")
}

/// Directory a model is stored in below `cache_root`.
pub fn model_dir(cache_root: &Path, spec: &ModelSpec) -> PathBuf {
    cache_root.join(&spec.id)
}

/// Download the model into `<cache_root>/<id>` unless it is already complete.
/// `log` receives human readable progress lines. Returns the model directory.
pub fn fetch(
    spec: &ModelSpec,
    cache_root: &Path,
    options: &FetchOptions,
    log: &mut dyn FnMut(&str),
) -> Result<PathBuf> {
    let dir = model_dir(cache_root, spec);
    let stamp_path = dir.join(STAMP_FILE);
    if let Some(stamp) = fs::read_to_string(&stamp_path)
        .ok()
        .and_then(|s| serde_json::from_str::<Stamp>(&s).ok())
    {
        let complete = stamp.repo == spec.repo
            && stamp.revision == spec.revision
            && stamp.include == spec.include
            && stamp.files.iter().all(|f| dir.join(f).is_file());
        if complete {
            log(&format!(
                "model `{}` already present in {}",
                spec.id,
                dir.display()
            ));
            return Ok(dir);
        }
    }

    let endpoint = options
        .endpoint
        .as_deref()
        .unwrap_or(DEFAULT_ENDPOINT)
        .trim_end_matches('/');
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(30)))
        .build()
        .into();
    let get = |url: &str| -> Result<ureq::http::Response<ureq::Body>> {
        let mut request = agent.get(url);
        if let Some(token) = &options.token {
            request = request.header("Authorization", &format!("Bearer {token}"));
        }
        request.call().map_err(|e| format!("GET {url}: {e}"))
    };

    let rev = encode_segment(&spec.revision);
    let info_url = format!(
        "{endpoint}/api/models/{}/revision/{rev}?blobs=true",
        spec.repo
    );
    let info: RevisionInfo = get(&info_url)?
        .body_mut()
        .read_json()
        .map_err(|e| format!("parse {info_url}: {e}"))?;

    let files: Vec<&Sibling> = info
        .siblings
        .iter()
        .filter(|s| is_wanted(&s.rfilename, &spec.include))
        .collect();
    if !files.iter().any(|f| f.rfilename == "config.json") {
        return Err(format!(
            "{} @ {} has no config.json matching {:?}; check `repo`/`revision`/`include`",
            spec.repo, spec.revision, spec.include
        ));
    }
    if !files.iter().any(|f| f.rfilename.ends_with(".safetensors")) {
        return Err(format!(
            "{} @ {} has no *.safetensors weights matching {:?}",
            spec.repo, spec.revision, spec.include
        ));
    }

    fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    // A previous partial/other-revision download must not leak into this one.
    let _ = fs::remove_file(&stamp_path);

    for (i, file) in files.iter().enumerate() {
        let target = dir.join(&file.rfilename);
        if let (Some(size), Ok(meta)) = (file.size, fs::metadata(&target)) {
            if meta.len() == size {
                log(&format!(
                    "[{}/{}] {} (cached)",
                    i + 1,
                    files.len(),
                    file.rfilename
                ));
                continue;
            }
        }
        log(&format!(
            "[{}/{}] downloading {}",
            i + 1,
            files.len(),
            file.rfilename
        ));
        let url = format!(
            "{endpoint}/{}/resolve/{rev}/{}",
            spec.repo,
            encode_segment(&file.rfilename)
        );
        let part = dir.join(format!("{}.part", file.rfilename));
        let mut response = get(&url)?;
        let mut reader = response.body_mut().as_reader();
        let mut out =
            fs::File::create(&part).map_err(|e| format!("create {}: {e}", part.display()))?;
        let written = io::copy(&mut reader, &mut out)
            .map_err(|e| format!("download {}: {e}", file.rfilename))?;
        out.flush()
            .map_err(|e| format!("flush {}: {e}", part.display()))?;
        if let Some(size) = file.size {
            if written != size {
                let _ = fs::remove_file(&part);
                return Err(format!(
                    "{}: expected {size} bytes, got {written}",
                    file.rfilename
                ));
            }
        }
        fs::rename(&part, &target).map_err(|e| format!("rename {}: {e}", part.display()))?;
    }

    // Fail the build now rather than at the first chat message, minutes after load.
    if !has_chat_template(&dir) {
        return Err(format!(
            "{} @ {} has no chat template (chat_template.jinja, chat_template.json or a \
             `chat_template` in tokenizer_config.json) among the files matching {:?}",
            spec.repo, spec.revision, spec.include
        ));
    }

    let stamp = Stamp {
        repo: spec.repo.clone(),
        revision: spec.revision.clone(),
        include: spec.include.clone(),
        commit: info.sha,
        files: files.iter().map(|f| f.rfilename.clone()).collect(),
    };
    let json = serde_json::to_string_pretty(&stamp).map_err(|e| e.to_string())?;
    fs::write(&stamp_path, json).map_err(|e| format!("write {}: {e}", stamp_path.display()))?;
    Ok(dir)
}

#[cfg(test)]
mod tests;
