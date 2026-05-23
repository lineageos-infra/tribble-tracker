FROM rust:1.95 AS chef 
RUN cargo install --locked cargo-chef 
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin tribble-tracker-rs

FROM debian:13.5-slim
RUN apt update && apt install -y curl sqlite3
COPY --from=builder /app/target/release/tribble-tracker-rs /usr/local/bin
ENTRYPOINT ["tribble-tracker-rs"]
