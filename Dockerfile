# Multi-stage build for optimized, secure production image
# Stage 1: Build stage with full Rust toolchain
FROM rust:1.75-slim as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user for building
RUN useradd -m -u 1001 appuser

# Set working directory
WORKDIR /usr/src/imaginary-rs

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && \
    rm -rf src/

# Copy actual source code
COPY src/ src/
COPY tests/ tests/
COPY config/ config/

# Build the application
RUN cargo build --release --bin imaginary-rs

# Stage 2: Runtime stage with minimal distroless image
FROM gcr.io/distroless/cc-debian12:nonroot

# Copy SSL certificates for HTTPS requests
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the built binary
COPY --from=builder /usr/src/imaginary-rs/target/release/imaginary-rs /usr/local/bin/imaginary-rs

# Copy configuration files
COPY --from=builder /usr/src/imaginary-rs/config/ /usr/local/etc/imaginary-rs/config/

# Set the user to nonroot (uid 65532)
USER nonroot:nonroot

# Expose the application port
EXPOSE 8080

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD ["/usr/local/bin/imaginary-rs", "--health-check"] || exit 1

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/imaginary-rs"]
CMD ["--config", "/usr/local/etc/imaginary-rs/config/default.toml"]
