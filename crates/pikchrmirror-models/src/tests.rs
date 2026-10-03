use super::*;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

const REGISTRY: &str = r#"
selected = "granite"

[models.granite]
name = "Granite"
repo = "org/granite"

[models.apertus]
name = "Apertus"
repo = "org/apertus"
supported = false
reason = "no loader"
"#;

fn spec() -> ModelSpec {
    Registry::parse(REGISTRY).unwrap().models["granite"].clone()
}

#[test]
fn parse_fills_ids_and_defaults() {
    let registry = Registry::parse(REGISTRY).unwrap();
    let granite = &registry.models["granite"];
    assert_eq!(granite.id, "granite");
    assert_eq!(granite.revision, "main");
    assert!(granite.supported);
    assert!(granite.include.contains(&"*.safetensors".to_string()));
}

#[test]
fn resolve_uses_selected_override_and_none() {
    let registry = Registry::parse(REGISTRY).unwrap();
    assert!(matches!(
        registry.resolve(None).unwrap(),
        Selection::Model(m) if m.id == "granite"
    ));
    assert!(matches!(
        registry.resolve(Some("  ")).unwrap(),
        Selection::Model(m) if m.id == "granite"
    ));
    assert_eq!(registry.resolve(Some("NONE")).unwrap(), Selection::Disabled);
}

#[test]
fn resolve_rejects_unknown_and_unsupported() {
    let registry = Registry::parse(REGISTRY).unwrap();
    let unknown = registry.resolve(Some("gpt")).unwrap_err();
    assert!(unknown.contains("granite") && unknown.contains("apertus"));
    let unsupported = registry.resolve(Some("apertus")).unwrap_err();
    assert!(unsupported.contains("no loader"));
}

#[test]
fn wildcard_matching() {
    assert!(matches_pattern("*.json", "config.json"));
    assert!(matches_pattern(
        "model-*-of-*.safetensors",
        "model-00001-of-00002.safetensors"
    ));
    assert!(matches_pattern("tokenizer.model", "tokenizer.model"));
    assert!(!matches_pattern("*.json", "config.json.bak"));
    assert!(!matches_pattern("tokenizer.model", "tokenizer.model2"));
    assert!(!matches_pattern("a*a", "a"));
}

#[test]
fn wanted_files_are_top_level_and_safe() {
    let include = default_include();
    assert!(is_wanted("config.json", &include));
    assert!(!is_wanted("onnx/model.json", &include));
    assert!(!is_wanted("../config.json", &include));
    assert!(!is_wanted(".hidden.json", &include));
    assert!(!is_wanted("model.gguf", &include));
}

#[test]
fn segment_encoding() {
    assert_eq!(encode_segment("refs/pr/1"), "refs%2Fpr%2F1");
    assert_eq!(encode_segment("main"), "main");
}

/// Minimal Hugging Face lookalike. Serves one revision listing and the files,
/// counting every request so tests can assert on network use.
struct FakeHub {
    endpoint: String,
    requests: Arc<AtomicUsize>,
}

fn fake_hub(files: Vec<(&'static str, Vec<u8>)>, lie_about_size: bool) -> FakeHub {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(AtomicUsize::new(0));
    let counter = requests.clone();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            counter.fetch_add(1, Ordering::SeqCst);
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap() > 2 {
                line.clear();
            }
            let path = request_line.split_whitespace().nth(1).unwrap_or("");
            let (status, body): (&str, Vec<u8>) =
                if path.starts_with("/api/models/org/granite/revision/main") {
                    let siblings: Vec<String> = files
                        .iter()
                        .map(|(n, b)| {
                            let size = b.len() + usize::from(lie_about_size);
                            format!(r#"{{"rfilename":"{n}","size":{size}}}"#)
                        })
                        .chain([r#"{"rfilename":"model.gguf","size":1}"#.to_string()])
                        .collect();
                    (
                        "200 OK",
                        format!(r#"{{"sha":"abc123","siblings":[{}]}}"#, siblings.join(","))
                            .into_bytes(),
                    )
                } else if let Some(name) = path.strip_prefix("/org/granite/resolve/main/") {
                    match files.iter().find(|(n, _)| *n == name) {
                        Some((_, b)) => ("200 OK", b.clone()),
                        None => ("404 Not Found", Vec::new()),
                    }
                } else {
                    ("404 Not Found", Vec::new())
                };
            let head = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(&body);
            let mut sink = Vec::new();
            let _ = stream.read_to_end(&mut sink);
        }
    });
    FakeHub { endpoint, requests }
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pikchr-models-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn sample_files() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("config.json", b"{}".to_vec()),
        ("model.safetensors", vec![7u8; 4096]),
    ]
}

#[test]
fn fetch_downloads_filtered_files_then_skips_network() {
    let hub = fake_hub(sample_files(), false);
    let cache = temp_dir("ok");
    let options = FetchOptions {
        endpoint: Some(hub.endpoint.clone()),
        token: None,
    };

    let dir = fetch(&spec(), &cache, &options, &mut |_| {}).unwrap();
    assert_eq!(dir, cache.join("granite"));
    assert_eq!(fs::read(dir.join("config.json")).unwrap(), b"{}");
    assert_eq!(fs::read(dir.join("model.safetensors")).unwrap().len(), 4096);
    assert!(
        !dir.join("model.gguf").exists(),
        "excluded by include patterns"
    );
    assert!(!dir.join("model.safetensors.part").exists());

    let after_first = hub.requests.load(Ordering::SeqCst);
    assert_eq!(after_first, 3, "listing + two files");
    fetch(&spec(), &cache, &options, &mut |_| {}).unwrap();
    assert_eq!(
        hub.requests.load(Ordering::SeqCst),
        after_first,
        "complete stamp => offline"
    );
    let _ = fs::remove_dir_all(&cache);
}

#[test]
fn fetch_refetches_when_revision_changes() {
    let hub = fake_hub(sample_files(), false);
    let cache = temp_dir("rev");
    let options = FetchOptions {
        endpoint: Some(hub.endpoint.clone()),
        token: None,
    };
    fetch(&spec(), &cache, &options, &mut |_| {}).unwrap();
    let before = hub.requests.load(Ordering::SeqCst);

    let mut other = spec();
    other.include.push("*.md".into());
    fetch(&other, &cache, &options, &mut |_| {}).unwrap();
    assert!(hub.requests.load(Ordering::SeqCst) > before);
    let _ = fs::remove_dir_all(&cache);
}

#[test]
fn fetch_detects_truncated_download() {
    let hub = fake_hub(sample_files(), true);
    let cache = temp_dir("size");
    let options = FetchOptions {
        endpoint: Some(hub.endpoint.clone()),
        token: None,
    };
    let err = fetch(&spec(), &cache, &options, &mut |_| {}).unwrap_err();
    assert!(err.contains("expected"), "{err}");
    assert!(!cache.join("granite").join(STAMP_FILE).exists());
    let _ = fs::remove_dir_all(&cache);
}

#[test]
fn fetch_requires_config_and_weights() {
    let hub = fake_hub(vec![("README.md", b"x".to_vec())], false);
    let cache = temp_dir("missing");
    let options = FetchOptions {
        endpoint: Some(hub.endpoint.clone()),
        token: None,
    };
    let err = fetch(&spec(), &cache, &options, &mut |_| {}).unwrap_err();
    assert!(err.contains("config.json"), "{err}");
    let _ = fs::remove_dir_all(&cache);
}

#[test]
fn fetch_reports_http_errors() {
    let hub = fake_hub(sample_files(), false);
    let cache = temp_dir("404");
    let options = FetchOptions {
        endpoint: Some(hub.endpoint.clone()),
        token: None,
    };
    let mut missing = spec();
    missing.repo = "org/missing".into();
    let err = fetch(&missing, &cache, &options, &mut |_| {}).unwrap_err();
    assert!(err.contains("404"), "{err}");
    let _ = fs::remove_dir_all(&cache);
}
