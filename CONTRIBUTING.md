# Contributing to Imaginary-rs

Thank you for your interest in contributing!

## Documentation Requirements
- **Update documentation with every feature, bugfix, or breaking change.**
  - This includes the README, CLI help, and relevant code comments.
- **Follow best practices:**
  - Define the scope and audience for your documentation
  - Use clear, concise language
  - Provide examples and usage instructions
  - Regularly review and update documentation as code evolves
  - Validate documentation for accuracy and completeness
  - Collaborate with the team to keep docs up-to-date
- See [LinkedIn best practices for documentation](https://www.linkedin.com/advice/0/what-best-practices-keeping-your-software-documentation-28sje)

## Code Guidelines
- Write clear, maintainable, and well-tested code
- Follow the existing code style and structure
- Add or update tests for new features and bugfixes

## Pull Requests
- Ensure all tests pass
- Ensure documentation is updated and accurate
- Provide a clear description of your changes

Thank you for helping keep Imaginary-rs high quality and well-documented!

## Vertical Modular Structure

- All image operations are organized in a deep, vertical module tree under [`src/image/operations/`](src/image/operations/).
- Each logical group of operations (e.g., transform, color, format, overlay, watermark) has its own submodule.
- When adding a new operation, create a new submodule if needed to keep files focused and under 300 lines.

## File Size Policy

- **Target:** Keep all source files under 300 lines.
- **Refactor:** If a file approaches 270 lines, refactor by splitting into deeper submodules.
- **Rationale:** This ensures maintainability, discoverability, and ease of review.

## Adding New Operations

1. Identify the appropriate submodule (or create a new one).
2. Implement the operation as a public function with a clear doc comment and usage example.
3. Add unit tests in the same file as the operation.
4. Update the module-level doc comment to include the new operation.
5. Re-export the operation in `mod.rs` if it should be part of the public API.
6. Update the README and `test.html` if the operation is user-facing.

## Testing

- Every operation must have unit tests covering normal and edge cases.
- Place tests in a `#[cfg(test)] mod tests` section at the bottom of the file.
- Run `cargo test --all` before submitting a pull request.

## Documentation

- Every public function, struct, and module must have a `///` doc comment.
- Include a `# Examples` section for at least one function per file.
- Update the README with new operations and usage examples as needed.

## Code Style

- Follow Rust's standard formatting (`cargo fmt`).
- Use clear, descriptive names and idiomatic Rust patterns.
- Minimize the public API surface; use `pub(crate)` or private visibility for helpers.
- Prefer vertical, modular structure over large, flat files.

## Command Line Options

- `--concurrency <N>`: Maximum number of concurrent HTTP requests to process (0 = unlimited, default: 0). Matches the original imaginary's concurrency option.

---

Thank you for helping make Imaginary-rs robust, maintainable, and a model of engineering excellence! 