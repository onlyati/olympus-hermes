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

FROM debian:bullseye-slim
ARG APP=/usr/local/bin

RUN apt-get update && \
    apt-get install -y ca-certificates tzdata coreutils locales && \
    rm -rf /var/lib/apt/lists/*

EXPOSE 3030
EXPOSE 3031
EXPOSE 3032

ENV RUST_LOG=info

COPY ./main_default.conf /etc/defaults/hermes.conf

COPY --from=builder /rust-docker/target/release/hermes ${APP}/hermes
COPY --from=builder /rust-docker/target/release/cli ${APP}/cli

RUN mkdir -p "/usr/var/hermes" && echo "" > /usr/var/hermes/init.dat \
    && mkdir -p "/usr/var/hermes" && echo "" > /usr/var/hermes/hook.dat \
    && mkdir -p "/etc/hermes" && cp "/etc/defaults/hermes.conf" "/etc/hermes/main.conf"

VOLUME /usr/var/hermes
VOLUME /etc/hermes

WORKDIR ${APP}

ENTRYPOINT ./hermes /etc/hermes/main.conf
