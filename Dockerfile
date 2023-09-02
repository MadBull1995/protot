FROM rust:latest

WORKDIR /app

# Install protobuf compiler
RUN apt-get update && \
    apt-get install -y protobuf-compiler

COPY ./ ./

RUN cargo build --release --features "stats"

EXPOSE 44880

CMD ["./target/release/proto_tasker"]