FROM rust:bookworm as builder

RUN apt-get update && \
    apt-get install -y protobuf-compiler

WORKDIR /
RUN cargo new --bin rust-docker
WORKDIR /rust-docker

RUN rm src/*.rs
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./crates ./crates

RUN cargo update \
    && cargo build --release

FROM debian:bookworm-slim
ARG APP=/usr/local/bin

RUN apt-get update && \
    apt-get install -y ca-certificates tzdata coreutils locales curl libssl-dev && \
    rm -rf /var/lib/apt/lists/*

EXPOSE 3031
EXPOSE 3032
EXPOSE 3033

ENV HERMES_LOG=info
ENV HERMES_CLI_LOG=error

COPY ./config_default.toml /etc/defaults/hermes.toml

COPY --from=builder /rust-docker/target/release/hermes ${APP}/hermes

RUN mkdir -p "/usr/var/hermes" && echo "" > /usr/var/hermes/init.dat \
    && mkdir -p "/usr/var/hermes" && echo "" > /usr/var/hermes/hook.dat \
    && mkdir -p "/etc/olympus/hermes" && cp "/etc/defaults/hermes.toml" "/etc/olympus/hermes/config.toml"

VOLUME /usr/var/hermes
VOLUME /etc/olympus/hermes

HEALTHCHECK --interval=10s --timeout=3s --start-period=2s --retries=3 \
  CMD curl --fail --connect-timeout 5 127.0.0.1:3032/hc || exit 1

WORKDIR ${APP}

CMD [ "./hermes", "server", "-c", "/etc/olympus/hermes/config.toml" ]
