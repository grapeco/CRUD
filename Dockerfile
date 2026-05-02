FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && \
    apt-get install -y libssl3
COPY --from=builder /app/target/release/server .

EXPOSE 8080
CMD ["./server"]