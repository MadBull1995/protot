FROM rust:latest as build

# create a new empty shell project
RUN USER=root cargo new --bin protot
WORKDIR /protot
# Install protobuf compiler
RUN apt-get update && \
    apt-get install -y protobuf-compiler

# copy over manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache dependencies
RUN cargo build --release --features "stats"
RUN rm src/*.rs

# Copy core files for release
COPY ./src ./src
COPY ./protos ./protos
COPY ./build.rs ./build.rs


# build for release
RUN rm ./target/release/deps/protot*
RUN cargo build --release --features "stats"

# our final base
FROM rust:latest
WORKDIR /protot
# Install protobuf compiler
RUN apt-get update && \
    apt-get install -y protobuf-compiler

# copy the build artifact from the build stage
COPY --from=build /protot/target/release/protot .
COPY ./configs.yaml ./configs.yaml
CMD ["./protot init --data-host redis://redis/ --grpc-port 44880"]