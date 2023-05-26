FROM rust:latest as builder

RUN apt-get update && \
    apt-get install -y protobuf-compiler

RUN cargo new --bin rust-docker
WORKDIR ./rust-docker

RUN rm src/*.rs
COPY ./proto ./proto
COPY ./src ./src
COPY ./build.rs ./build.rs
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release

FROM debian
ARG APP=/usr/local/bin

RUN apt-get update && \
    apt-get install -y ca-certificates tzdata coreutils locales && \
    rm -rf /var/lib/apt/lists/*

EXPOSE 3000
EXPOSE 3001
EXPOSE 3002

ENV RUST_LOG=info

COPY ./main_default.conf /etc/hermes/main.conf
COPY ./docker/entry.sh ${APP}/entry.sh
RUN chmod +x ${APP}/entry.sh

COPY --from=builder /rust-docker/target/release/hermes ${APP}/hermes
COPY --from=builder /rust-docker/target/release/cli ${APP}/cli

WORKDIR ${APP}

# ENTRYPOINT "./entry.sh"
ENTRYPOINT ["/bin/sh", "entry.sh"]
