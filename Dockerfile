FROM messense/rust-musl-cross:x86_64-musl as chef_exchange
RUN cargo install cargo-chef
WORKDIR /app

FROM chef_exchange AS planner_exchange
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef_exchange AS builder_exchange
COPY --from=planner_exchange /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
WORKDIR /app
COPY --from=builder_exchange /app/target/x86_64-unknown-linux-musl/release/rs-exchanges-parser rs-exchanges-parser
ENTRYPOINT ["RUST_LOG=info /app/rs-exchanges-parser"]