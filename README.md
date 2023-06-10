# Hermes@Olympus

Hermes is a message queue and in-memory database systems that I use for my hobbi projects. Interfaces are built top of [datastore-rs](https://git.thinkaboutit.tech/PublicProjects/datastore-rs) library. It has all abilities to:
- Set/Get/Remove keys
- List keys
- Set/Get/Remove/Trigger web hooks
- Suspend/Resume its logging
- Running stored procedures in form of lua scripts (lua 5.4 is supported)+

For detailed information read [documentation](./docs/README.md).

## Example for usage

Hermes bascially has three separated interfaces and a CLI:
1. Simple TCP based interface
2. gRPC based interface
3. REST based interface
4. Bash CLI utility (that is using gRPC interface internally)

Any command can be executed on any interface, for example:
```bash
# Using the simple TCP interface
echo -n "GET /root/status" | socat - tcp:127.0.0.1:3030

# Using the gRPC interface via command line utility
hermes cli -H http://127.0.0.1:3031 get --key "/root/status"

# Using the REST API via cURL
curl "127.0.0.1:3032/db?key=/root/status"
```

For more details, check the [docs](docs/README.md).

## Install manually

To install you need to have Rust package installed installed with 1.68 minimal version. You also have to have protobuf-compiler package installed.
Then you have to perform the following steps:
```bash
git clone https://git.thinkaboutit.tech/PublicProjects/olympus-hermes
cd olympus-hermes/hermes
cargo build --release
```

After it, executable binary (`target/release/hermes`) is built. You can start server by `hermes server -c <path-to-config>` command. Sample config file can be found in [config_default.toml](hermes/config_default.toml) file.

## Install via docker

Dockerfile and docker-compose file are prepare. It is also possible to build hermes with docker file:
```bash
git clone https://git.thinkaboutit.tech/PublicProjects/olympus-hermes
cd olympus-hermes/hermes
docker build -t hermes .
```

Wait until it is done, it can take a few minutes. When it is done image must appear among local images. After reviewed the `docker/docker-compose.yaml` file and prepared the volumes, it can be started:
```bash
cd docker
docker-compose up -d
```

