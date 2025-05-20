# imaginary-rs

A Rust implementation of the [h2non/imaginary](https://github.com/h2non/imaginary) image processing service.

## Features

- HTTP server for high-level image processing
- Flexible image manipulation pipeline via `/pipeline` endpoint
- Security middleware (API key, CORS)
- Configurable via file, env, or CLI
- Extensible: add new operations easily

## Supported Operations (for pipeline)

- `resize`: Resize an image (params: `width`, `height`)
- `crop`: Crop an image (params: `x`, `y`, `width`, `height`)
- `rotate`: Rotate image (params: `degrees`)
- `grayscale`: Convert to grayscale (no params)
- `blur`: Blur image (params: `sigma`)
- `flip`: Flip vertically (no params)
- `flop`: Flip horizontally (no params)
- `adjustBrightness`: Adjust brightness (params: `value`)
- `adjustContrast`: Adjust contrast (params: `value`)
- `sharpen`: Sharpen image (no params)
- `convert`: Change format (params: `format`, `quality`)
- ...and more (see code for full list)

## API Endpoints

### POST /pipeline
Process an image with a sequence of operations.

**Request:** `multipart/form-data`
- `image`: The image file
- `operations`: JSON array of operation specs, e.g.

```
[
  {"operation": "resize", "params": {"width": 200, "height": 200}},
  {"operation": "grayscale", "params": {}}
]
```

**Response:** Processed image (binary)

### POST /process (legacy)
Legacy endpoint for single-operation (resize) processing.

### POST /convert (legacy)
Legacy endpoint for format conversion.

### GET /health
Health check.

## Usage Example

See `test.html` for a browser-based demo.

## Building & Running

```sh
cargo build --release
cargo run
```

## Contributing: Adding New Operations

1. Implement the operation in `src/image/operations.rs`.
2. Add parameter struct and validation in `src/image/params.rs`.
3. Add to `SupportedOperation` in `src/image/pipeline_types.rs`.
4. Update `execute_single_operation` in `src/image/pipeline_executor.rs`.
5. Add tests.
6. Document the operation in this README.

---
MIT License.