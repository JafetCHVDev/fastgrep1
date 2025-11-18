FROM rust:1.72-slim-bullseye AS builder
WORKDIR /usr/src/fastgrep
COPY . .
RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential git && \
    cargo install --path . --locked --root /usr/local

FROM debian:bullseye-slim
COPY --from=builder /usr/local/bin/fastgrep /usr/local/bin/fastgrep
ENTRYPOINT ["/usr/local/bin/fastgrep"]
