FROM rust:latest as builder
WORKDIR /usr/src/rust-stream
COPY . .
RUN cargo build --release --bin consumer


FROM ubuntu:24.04
WORKDIR /usr/local/bin

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/rust-stream/target/release/consumer .

CMD ["./consumer"]