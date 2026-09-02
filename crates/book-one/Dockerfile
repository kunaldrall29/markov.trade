# book-one — the house agent. Built from the workspace root:
#   docker build -f crates/book-one/Dockerfile -t book-one .
# Shadow mode only in this build (P08). No keypair is copied into the image;
# in devnet mode (P07) the operator key arrives via the service environment.
FROM rust:1.97.1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY programs ./programs
RUN cargo build --release -p book-one --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd --system --uid 10001 --create-home book
COPY --from=build /src/target/release/book-one /usr/local/bin/book-one
USER book
ENV VENUE=shadow \
    TICK_SECONDS=60 \
    MARK_SOURCE=onchain \
    PAPER_DIR=/data/paper \
    RUST_LOG=info
VOLUME ["/data/paper"]
ENTRYPOINT ["/usr/local/bin/book-one"]
