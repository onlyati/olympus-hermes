FROM rust:latest as builder

RUN apt-get update && \
    apt-get install -y protobuf-compiler

WORKDIR /
RUN cargo new --bin rust-docker
WORKDIR /rust-docker

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

ENV HERMES_LOG=info

COPY ./config_default.toml /etc/defaults/hermes.toml

COPY --from=builder /rust-docker/target/release/hermes ${APP}/hermes
COPY --from=builder /rust-docker/target/release/cli ${APP}/cli

RUN mkdir -p "/usr/var/hermes" && echo "" > /usr/var/hermes/init.dat \
    && mkdir -p "/usr/var/hermes" && echo "" > /usr/var/hermes/hook.dat \
    && mkdir -p "/etc/olympus/hermes" && cp "/etc/defaults/hermes.toml" "/etc/olympus/hermes/config.toml"

VOLUME /usr/var/hermes
VOLUME /etc/olympus/hermes

WORKDIR ${APP}

CMD [ "./hermes", "/etc/olympus/hermes/config.toml" ]
