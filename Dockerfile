FROM rust:1.68 as builder
WORKDIR /usr/src/imaginary-rs
COPY . .
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /usr/src/imaginary-rs/target/release/imaginary-rs /usr/local/bin/
EXPOSE 8080
CMD ["imaginary-rs"]
