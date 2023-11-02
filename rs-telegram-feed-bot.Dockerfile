FROM messense/rust-musl-cross:x86_64-musl as chef_telegram
RUN cargo install cargo-chef
WORKDIR /app

FROM chef_telegram AS planner_telegram
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef_telegram AS builder_telegram
COPY --from=planner_telegram /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.14
WORKDIR /app
ENV RUST_LOG info
RUN touch /app/.env
COPY --from=builder_telegram /app/target/x86_64-unknown-linux-musl/release/rs-telegram-feed-bot /app/rs-telegram-feed-bot
ENTRYPOINT ["/app/rs-telegram-feed-bot"]