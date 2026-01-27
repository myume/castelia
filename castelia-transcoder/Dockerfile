FROM rust:1.91.0 AS builder
WORKDIR /app
COPY . .
RUN SQLX_OFFLINE=true cargo build --release --package castelia-transcoder

FROM debian:trixie-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ffmpeg && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/castelia-transcoder /app/castelia-transcoder

CMD ["/app/castelia-transcoder"]

