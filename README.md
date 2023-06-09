# Hermes@Olympus

Hermes is a message queue and in-memory database systems that I use for my hobbi projects. Interfaces are built top of [datastore-rs](https://git.thinkaboutit.tech/PublicProjects/datastore-rs) library. It has all abilities to:
- Set/Get/Remove keys
  - Value type of key's value can be string or a string queue (FIFO)
- List keys
- Set/Get/Remove/Trigger web hooks
- Suspend/Resume its logging
- Running stored procedures in form of lua scripts (lua 5.4 is supported)
- Receive Gitea hooks and process them

For detailed information read [documentation](./docs/README.md).

## Example for usage

Hermes bascially has three separated interfaces and a CLI:
1. Simple TCP based interface
1. REST based interface
1. Websocket based interface
1. Command line interface (websocket based)

Any command can be executed on any interface, for example:
```bash
# Using the simple TCP interface
$ echo -n "GET /hermes1/status" | socat - tcp:127.0.0.1:3030
>Ok
running

# Using via command line utility
$ hermes cli -H ws://127.0.0.1:3034 get --key "/hermes1/status"
running

# Using the REST API via cURL
$ curl "127.0.0.1:3032/db?key=/hermes1/status"
"running"

# Using websocket
$ wscat -c "127.0.0.1:3033/ws" -x '{ "command" : "GetKey", "key" : "/hermes1/status" }'
{"status":"Ok","message":"running"}
```

There is also a shell that can be used to execute any command, example for usage:
```
$ hermes shell -c ./client_config.toml
hermes@disconnected=> \l
dev                 ws://127.0.0.1:3043
test                ws://127.0.0.1:3033
hermes@disconnected=> \c cfg://test
hermes@ws://127.0.0.1:3033=> list-keys -k /hermes1
/hermes1/status/proxy
/hermes1/status/server1
/hermes1/status/server2
/hermes1/status/server3
/hermes1/status/server4
/hermes1/test1
/hermes1/test2
hermes@ws://127.0.0.1:3033=> \c cfg://dev
hermes@ws://127.0.0.1:3043=> list-keys -k /hermes1
/hermes1/ati/hooks/status1
/hermes1/ati/test
/hermes1/ati/test-script
/hermes1/status/server1
hermes@ws://127.0.0.1:3043=> \d
hermes@disconnected=> \q
```

For more details, check the [docs](docs/README.md).

## Docker install

Easiest way to install Hermes is using docker, pull the image by `docker pull onlyati/hermes` and based on [docker-compose](hermes/docker/docker-compose.yaml) it can be started easily.

## Build yourself manually

To install you need to have Rust package installed installed with 1.68 minimal version. You also have to have protobuf-compiler package installed.
Then you have to perform the following steps:
```bash
$ git clone https://git.thinkaboutit.tech/PublicProjects/olympus-hermes
$ cd olympus-hermes/hermes
$ cargo update
$ cargo build --release
```

After it, executable binary (`target/release/hermes`) is built. You can start server by `hermes server -c <path-to-config>` command. Sample config file can be found in [config_default.toml](hermes/config_default.toml) file.

## Build yourself with Docker

Dockerfile and docker-compose file are prepare. It is also possible to build hermes with docker file:
```bash
$ git clone https://git.thinkaboutit.tech/PublicProjects/olympus-hermes
$ cd olympus-hermes/hermes
$ docker build -t hermes .
```

Wait until it is done, it can take a few minutes. When it is done image must appear among local images. After reviewed the `docker/docker-compose.yaml` file and prepared the volumes, it can be started:
```bash
$ cd docker
$ docker compose up -d
```

