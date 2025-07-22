# imaginary-rs

A Rust implementation of the [h2non/imaginary](https://github.com/h2non/imaginary) image processing service.

## Features

- HTTP server for high-level image processing
- Flexible image manipulation pipeline via `/pipeline` endpoint
- **NEW**: GET request support for `/pipeline` endpoint with URL-based image fetching
- **NEW**: Enhanced format handling - defaults to original image format unless convert operation specified
- Security middleware (API key, CORS)
- Configurable via file, env, or CLI
- Extensible: add new operations easily
- HTTP/1.1 and HTTP/2 support (user-selectable)
- Flexible TLS: self-signed or user-provided certificates
- Automatic self-signed certificate generation if missing
- HTTP/2 mode: HTTPS on port 3000, HTTP/1.1 redirect on 8080
- HTTP/1.1 mode: HTTP on port 8080 (default)
- All endpoints, logging, and middleware preserved
- **NEW**: Comprehensive test coverage with 71+ unit tests

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

### GET /pipeline
**NEW**: Process an image from a URL with a sequence of operations.

**Request Parameters:**
- `url`: URL of the image to process (HTTP/HTTPS only)
- `operations`: JSON-encoded array of operation specs

**Example:**
```
GET /pipeline?url=https://example.com/image.jpg&operations=[{"operation":"resize","params":{"width":200,"height":200}}]
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

1. Implement the operation in its own submodule under `src/image/operations/`.
2. Add parameter struct and validation in `src/image/params.rs`.
3. Add to `SupportedOperation` in `src/image/pipeline_types.rs`.
4. Update `execute_single_operation` in `src/image/pipeline_executor.rs`.
5. Add tests in the same file as the operation.
6. Document the operation in this README if it is part of the public API.

## Image Operations: Modular Structure

Imaginary-rs organizes all image processing operations into a deep, maintainable vertical module structure:

| Module      | Public Operations (re-exported at top level)                                         |
|-------------|--------------------------------------------------------------------------------------|
| `transform` | `resize`, `rotate`, `crop`, `flip_horizontal`, `flip_vertical`, `enlarge`, `extract`, `zoom`, `smart_crop`, `thumbnail` |
| `color`     | `grayscale`, `blur`, `adjust_brightness`, `adjust_contrast`, `sharpen`               |
| `format`    | `convert_format`, `autorotate`                                                       |
| `watermark` | `watermark`                                                                          |

All common operations are re-exported at the top level of the `operations` module for ergonomic use. Internal helpers (e.g., `overlay`, `draw_text`, `watermark_image`) are not part of the public API.

### Example: Using the Modular API in Rust

```rust
use imaginary::image::operations::{resize, grayscale, watermark, convert_format};
use imaginary::image::params::{ResizeParams, WatermarkParams, FormatConversionParams};

let img = /* Load a DynamicImage */;
let img = resize(img, &ResizeParams { width: 300, height: 300 });
let img = grayscale(img);
let img = watermark(img, &WatermarkParams {
    text: "Imaginary-rs".to_string(),
    opacity: 0.7,
    position: WatermarkPosition::BottomRight,
    font_size: 24,
    color: [0, 128, 255],
    x: None,
    y: None,
})?;
let img = convert_format(img, &FormatConversionParams {
    format: "jpeg".to_string(),
    quality: Some(85),
})?;
```

### Example: Using the HTTP API

Send a POST request to `/pipeline` with a multipart form containing:
- `image`: the image file
- `operations`: a JSON array of operations (see `test.html` for an example)

### Contributor Guide

- Each operation is implemented in its own submodule under [`src/image/operations/`](src/image/operations/).
- When adding new operations, create a new submodule if needed and keep files under 300 lines.
- Add unit tests in the same file as the operation.
- Only public, user-facing operations should be re-exported at the top level of the `operations` module.
- See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## Command Line Options

- `--concurrency <N>`: Maximum number of concurrent HTTP requests to process (0 = unlimited, default: 0). Matches the original imaginary's concurrency option.
- `--http-version <http1|http2>`: Select HTTP version (default: http1)
- `--tls-mode <self-signed|signed>`: TLS mode (default: self-signed)
- `--cert-path <PATH>`: Path to TLS certificate (default: cert.pem)
- `--key-path <PATH>`: Path to TLS private key (default: key.pem)

### Security Notes
- For production, always use a strong API key and salt
- Use signed certificates in production
- Self-signed certificates are for development/testing only
- **NEW**: URL fetching with comprehensive SSRF protection (hostname resolution, IP validation, private network blocking)

## Development Status

### ✅ Completed Features

- [x] Enhanced `/pipeline` endpoint with GET request support
- [x] URL-based image fetching with comprehensive SSRF protection
- [x] Improved format handling - defaults to original format unless convert operation specified
- [x] Comprehensive unit test coverage (71+ tests)
- [x] Parameter validation and error handling improvements
- [x] SOLID, CUPID, GRASP, SSOT, DRY, and ADP design principles implementation
- [x] All existing tests passing
- [x] Code cleanup and optimization

### 🔄 Current Development Stage: Complete

The next stage of development has been successfully completed with:

1. **Enhanced Pipeline Handler**: Added GET request support with URL fetching
2. **Improved Format Handling**: Smart format detection and preservation
3. **Comprehensive Testing**: 71+ unit tests covering all major functionality
4. **Code Quality**: Following best practices and design principles
5. **Security**: Comprehensive SSRF protection with IP validation and private network blocking

## Documentation Best Practices
- Documentation is updated with every major code change
- Scope and audience are defined for each doc section
- Examples and CLI usage are kept current
- [Best practices for documentation maintenance](https://www.linkedin.com/advice/0/what-best-practices-keeping-your-software-documentation-28sje):
  - Define scope and audience
  - Use clear, concise language
  - Update docs with code changes
  - Test and validate documentation
  - Foster a culture of documentation

---
MIT License.