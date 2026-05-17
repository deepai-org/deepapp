FROM rust:1.95-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:trixie-slim AS runtime

RUN useradd --create-home --shell /usr/sbin/nologin deepapp
COPY --from=builder /app/target/release/deepapp /usr/local/bin/deep
USER deepapp
WORKDIR /workspace

ENTRYPOINT ["deep"]
