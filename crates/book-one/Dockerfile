# book-one — the house agent. Built from the workspace root:
#   docker build -f crates/book-one/Dockerfile -t book-one .
# Two modes, one image. `VENUE=shadow` submits nothing; `VENUE=devnet` (P07)
# signs with the operator key, which arrives in `OPERATOR_KEY_JSON` from the
# service environment (`docs/14` §2). **No keypair is ever copied into the
# image**, and the process refuses to boot if it can read the emergency, owner,
# deployer or upgrade key.
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
    RUST_LOG=info \
    PORT=8080
# /health (503 when the last tick is older than three intervals) and /metrics.
EXPOSE 8080
# No VOLUME instruction: Railway rejects it ("use Railway Volumes"); the paper
# directory is a Railway volume mounted at /data/paper in the service config.
ENTRYPOINT ["/usr/local/bin/book-one"]
