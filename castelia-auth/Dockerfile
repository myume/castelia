FROM rust:1.91.0 as builder
WORKDIR /app
COPY . .
RUN SQLX_OFFLINE=true cargo build --release --package castelia-auth

FROM debian:trixie-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/castelia-auth /app/castelia-auth
CMD ["/app/castelia-auth"]

