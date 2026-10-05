# LLM Model Weights

The optional local model (`--features llm`) runs inside the `pikchrmirror-mcp` crate. It backs the `generate_pikchr` MCP tool and, through that crate, the desktop app's chat assistant. The weights
are **not** stored in git: they are downloaded **at build time** and the model
is a configuration item.

## Selecting the model

`models.toml` in the repository root is the registry. Its `selected` key picks
the model for every build:

```toml
selected = "qwen"
```

A single build can override it with an environment variable:

```sh
PIKCHR_MODEL=qwen cargo build --release -p pikchrmirror-mcp --features llm   # download + bundle
PIKCHR_MODEL=none    cargo check -p pikchrmirror-mcp --features llm          # compile only, no download
```

| id         | model                                   | status |
|------------|-----------------------------------------|--------|
| `qwen`     | Qwen2.5 0.5B Instruct                   | supported (default) |
| `granite`  | IBM Granite 4.0 H Micro (3B instruct)   | supported |

Inference uses [mistralrs](https://github.com/EricLBuehler/mistral.rs) 0.8.1, so a
model needs an architecture it can load. A table with `supported = false` and a
`reason` stops the build with that reason when selected.

To add a model, add a `[models.<id>]` table (`repo`, optionally `revision`,
`include`, `supported`). Pin `revision` to a commit hash for reproducible builds.

## What happens at build time

`crates/pikchrmirror-mcp/build.rs` (only with `--features llm`):

1. resolves the model from `PIKCHR_MODEL` or `models.toml`,
2. lists the repository through the Hugging Face API and downloads the config,
   tokenizer, chat template (`chat_template.jinja`, or inside `tokenizer_config.json`) and `*.safetensors` files into
   `target/pikchr-models/<id>/` (verifying each file size, resumable per file) and
   fails the build if the model ends up without a chat template,
3. skips the network entirely on later builds once the download is complete,
4. records the directory in the binary.

Build-time environment variables:

| variable             | purpose |
|----------------------|---------|
| `PIKCHR_MODEL`       | model id, or `none` to skip bundling |
| `PIKCHR_MODEL_CACHE` | download directory (default `target/pikchr-models`) |
| `HF_ENDPOINT`        | Hugging Face mirror (default `https://huggingface.co`) |
| `HF_TOKEN`           | access token for gated repositories |

The weights are *located*, not compiled into the executable: a 3B model is
several GB, which `include_bytes!` handles poorly. At runtime `pikchr-mcp` and the app look in
`PIKCHR_MODEL_DIR`, then the build-time directory. When shipping a relocated
build, set `PIKCHR_MODEL_DIR` to the copied model directory.

## Memory and speed

The model runs on the CPU in 32-bit floats (`f32`, candle's CPU matmul has no
BF16 kernel) without quantization. Qwen2.5 0.5B needs about 2 GB of RAM; a larger
model such as Granite 3B needs several times that.

To check a model directory end to end without the GUI (generates a diagram and asserts it compiles):

```sh
PIKCHR_MODEL_DIR=target/pikchr-models/qwen \
  cargo test -p pikchrmirror-mcp --features llm -- --ignored --nocapture
```

## Notes

- Without `--features llm` (the default) nothing is downloaded and no LLM code is built.
- `target/` and `*.gguf` are git-ignored.
