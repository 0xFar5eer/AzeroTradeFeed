FROM messense/rust-musl-cross:x86_64-musl as chef_subscan
RUN cargo install cargo-chef
WORKDIR /app

FROM chef_subscan AS planner_subscan
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef_subscan AS builder_subscan
COPY --from=planner_subscan /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.14
WORKDIR /app
ENV RUST_LOG info
RUN touch /app/.env
COPY --from=builder_subscan /app/target/x86_64-unknown-linux-musl/release/rs-subscan-parser /app/rs-subscan-parser
ENTRYPOINT ["/app/rs-subscan-parser"]