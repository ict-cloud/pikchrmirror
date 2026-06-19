# LLM Model Weights

This document describes how to acquire and set up LLM model weights for this project.

## Overview

The project uses GGUF format model files for local LLM inference. These files are not included in the repository due to their size, but can be downloaded and placed in the `assets/models/` directory.

## Acquiring Weights

Model weights in GGUF format can be obtained from:

1. **Hugging Face** - https://huggingface.co/
   - Search for GGUF quantized versions of models
   - Popular repositories include models from TheBloke and other quantization providers

2. **Ollama** - https://ollama.ai/
   - Pre-packaged models with automatic download
   - Recommended for ease of use

3. **Direct Downloads**
   - Some projects provide direct download links for GGUF files

## Setup

1. Download your preferred GGUF model files
2. Place them in the `assets/models/` directory
3. Update your application configuration to reference the model paths

## File Structure

```
assets/
└── models/
    └── (GGUF model files go here)
```

## Notes

- GGUF files are ignored by Git (see `.gitignore`)
- Models can be large (1GB - 50GB+), so ensure adequate disk space
- Different model sizes and quantization levels are available to suit your hardware constraints
