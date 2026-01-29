FROM rust:1.91.0 AS builder
WORKDIR /app

COPY . .

RUN SQLX_OFFLINE=true cargo build --release --package castelia-broadcast

FROM debian:trixie-slim
WORKDIR /app
EXPOSE 3001

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/castelia-broadcast /app/castelia-broadcast

CMD ["/app/castelia-broadcast"]
