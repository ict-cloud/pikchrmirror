//! Bundles the LLM weights at build time (feature `llm` only).
//!
//! Resolves the model from `models.toml` / `PIKCHR_MODEL`, downloads it into a
//! cache directory when missing and exports its location to the crate via
//! `PIKCHRMIRROR_MODEL_ID` / `PIKCHRMIRROR_MODEL_DIR`. With `PIKCHR_MODEL=none`
//! nothing is downloaded and both values are empty, so the crate still compiles
//! (the chat then reports that no model is bundled).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(feature = "llm")]
    llm::run();
    #[cfg(not(feature = "llm"))]
    println!("cargo:rustc-env=PIKCHRMIRROR_MODEL_ID=");
    #[cfg(not(feature = "llm"))]
    println!("cargo:rustc-env=PIKCHRMIRROR_MODEL_DIR=");
}

#[cfg(feature = "llm")]
mod llm {
    use pikchrmirror_models::{fetch, FetchOptions, Registry, Selection};
    use std::env;
    use std::path::PathBuf;

    pub fn run() {
        let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let registry_path = manifest.join("../../models.toml");
        println!("cargo:rerun-if-changed={}", registry_path.display());
        for var in ["PIKCHR_MODEL", "PIKCHR_MODEL_CACHE", "HF_ENDPOINT"] {
            println!("cargo:rerun-if-env-changed={var}");
        }

        let selection = Registry::load(&registry_path)
            .and_then(|r| r.resolve(env::var("PIKCHR_MODEL").ok().as_deref()))
            .unwrap_or_else(|e| fail(&e));

        let (id, dir) = match selection {
            Selection::Disabled => {
                println!("cargo:warning=PIKCHR_MODEL=none: building without a bundled LLM");
                (String::new(), String::new())
            }
            Selection::Model(spec) => {
                let options = FetchOptions {
                    endpoint: env::var("HF_ENDPOINT").ok().filter(|s| !s.is_empty()),
                    token: env::var("HF_TOKEN").ok().filter(|s| !s.is_empty()),
                };
                let dir = fetch(&spec, &cache_root(), &options, &mut |line| {
                    println!("cargo:warning={line}")
                })
                .unwrap_or_else(|e| fail(&e));
                (spec.id, dir.display().to_string())
            }
        };
        println!("cargo:rustc-env=PIKCHRMIRROR_MODEL_ID={id}");
        println!("cargo:rustc-env=PIKCHRMIRROR_MODEL_DIR={dir}");
    }

    /// `PIKCHR_MODEL_CACHE`, else `<target dir>/pikchr-models` (shared by all
    /// profiles, so debug and release builds download the weights once).
    fn cache_root() -> PathBuf {
        if let Some(dir) = env::var_os("PIKCHR_MODEL_CACHE").filter(|d| !d.is_empty()) {
            return PathBuf::from(dir);
        }
        // OUT_DIR = <target>/<profile>/build/<pkg>-<hash>/out
        let out = PathBuf::from(env::var("OUT_DIR").unwrap());
        out.ancestors().nth(4).unwrap_or(&out).join("pikchr-models")
    }

    fn fail(message: &str) -> ! {
        eprintln!(
            "\nerror: cannot bundle the LLM: {message}\n\
             hint: set PIKCHR_MODEL=none to build without a model, or pick another model in models.toml\n"
        );
        std::process::exit(1);
    }
}
