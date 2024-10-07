FROM rust:1.80 as base
WORKDIR /app
ADD . /app
RUN cargo build --release --bin server

FROM ubuntu:22.04
WORKDIR /app
RUN apt update && \
    apt install -y ca-certificates
COPY --from=builder /app/target/release/server /app
COPY --from=builder /app/config.toml /app
EXPOSE 8888
CMD ["/app/server"]