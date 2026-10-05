# LLM Model Weights

The optional chat assistant (`--features llm`) runs a local model. The weights
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
PIKCHR_MODEL=qwen cargo build --release --features llm   # download + bundle
PIKCHR_MODEL=none    cargo check --features llm              # compile only, no download
```

| id         | model                                   | status |
|------------|-----------------------------------------|--------|
| `qwen`     | Qwen2.5 0.5B Instruct                   | supported (default) |
| `granite`  | IBM Granite 4.0 H Micro (3B instruct)   | supported |
| `apertus`  | Swiss AI Apertus 8B Instruct            | registered, **not loadable yet** |
| `soofi`    | Soofi S Instruct (30B-A3B)              | registered, **not loadable yet** |

Inference uses [mistralrs](https://github.com/EricLBuehler/mistral.rs) 0.8.1.
It has a loader for IBM Granite 4.0 (`GraniteMoeHybrid`, safetensors) but none
for Apertus or Soofi, so selecting those stops the build with the reason. Soofi
S is also a 30B-parameter model, far too large to ship with a desktop app.
Support for them needs a newer mistralrs or another backend.

To add a model, add a `[models.<id>]` table (`repo`, optionally `revision`,
`include`, `supported`). Pin `revision` to a commit hash for reproducible builds.

## What happens at build time

`crates/pikchrmirror-app/build.rs` (only with `--features llm`):

1. resolves the model from `PIKCHR_MODEL` or `models.toml`,
2. lists the repository through the Hugging Face API and downloads the config,
   tokenizer, chat template (`chat_template.jinja`) and `*.safetensors` files into
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
several GB, which `include_bytes!` handles poorly. At runtime the app looks in
`PIKCHR_MODEL_DIR`, then the build-time directory, then `models/<id>/` next to
the executable. When shipping a relocated build, copy the model directory there.

## Memory and speed

The large layers are quantized to 4 bit when the model loads, on the CPU.
Some layers (e.g. Granite 4's Mamba projections) stay unquantized and run in
32-bit floats (`f32`) so they work with candle's CPU matmul. That means several
GB of RAM while loading and a slow first start.

`PIKCHR_MODEL_DTYPE` (runtime, default `f32`) overrides the precision. `bf16`
fails on the CPU with `unsupported dtype BF16 for op matmul`, and `f16` only works
on some platforms (not with Apple's Accelerate), so keep the default unless you
know your platform supports it.

To check a model directory end to end without the GUI:

```sh
PIKCHR_MODEL_DIR=target/pikchr-models/qwen \
  cargo test -p PikchrMirror --features llm -- --ignored --nocapture
```

## Notes

- Without `--features llm` (the default) nothing is downloaded and no LLM code is built.
- `target/` and `*.gguf` are git-ignored.
