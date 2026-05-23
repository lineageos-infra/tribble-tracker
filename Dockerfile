FROM rust:1.95-alpine3.23 AS chef
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

FROM node:24-alpine3.23 AS client
RUN npm install -g pnpm
WORKDIR /client
COPY client/package.json client/pnpm-lock.yaml client/pnpm-workspace.yaml .
RUN pnpm install --frozen-lockfile
COPY client .
RUN pnpm build

FROM alpine:3.23
RUN apk add --no-cache curl build-base tmux sqlite
WORKDIR /app
COPY --from=builder /app/target/release/tribble-tracker-rs /app/tribble-tracker-rs
COPY migrations /app/migrations
COPY --from=client /client/dist /app/client
ENTRYPOINT ["/app/tribble-tracker-rs"]
