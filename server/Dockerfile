FROM rust:1.82-slim

RUN cargo install cargo-watch

WORKDIR /app

COPY Cargo.toml ./
COPY crates/ crates/

RUN cargo build 2>/dev/null || true

EXPOSE 3001

CMD ["cargo", "watch", "-x", "run"]
