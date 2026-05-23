FROM rust:1.95-alpine AS chef
USER root
RUN cargo install --locked cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV SQLX_OFFLINE=true
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM alpine
RUN apk add --no-cache curl build-base tmux sqlite
WORKDIR /app
COPY --from=builder /app/target/release/tribble-tracker-rs /app/tribble-tracker-rs
COPY migrations /app/migrations
ENTRYPOINT ["/app/tribble-tracker-rs"]
