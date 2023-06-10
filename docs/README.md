# Hermes documents

## Actions and interfaces

Document has three major parts, they are discussing:
- [Actions](Actions.md)
- [Configuration](Configuration.md)
- [Command link interface](CommandLine.md)
- [Stored procedures](Stored_procedures.md)

Following documents describes each interface parameters:
- [Classic TCP interface](Interface_classic.md)
- [gRPC interface](Interface_gRPC.md)
- [REST interface](Interface_REST.md)

As Hermes is inteded to run on back-end server among other APIs and components, and not available directly from front-end applications. For this reason, all interfaces are unsecured to available more speed as possible. If, for some reason, any of these interfaces would be avaiable from front-end, it is handy to put it behind a proxy (e.g.: HAProxy) and setup security there.

## Applications logging

Application use environment variables for controllowing logging levels:
- `HERMES_LOG` for server
- `HERMES_CLI_LOG` for CLI

Log levels:
- trace
- debug
- info
- warn
- error

## Build and use Hermes on your own

Hermes is only one binary file. It does not need extra library to run or installing additional CLI utilities. Hermes's binary is called `hermes`.

### Via docker image

One option to start Hermes is to build a docker image and run it. [Dockerfile](../hermes/Dockerfile) can be used to build a Debian based image.
Then [docker-compose.yaml](../hermes/docker/docker-compose.yaml) file can be used to bring it up, after it has been supervised.

To build image yourself and run, execute the following commands:
```
$ git clone https://git.thinkaboutit.tech/PublicProjects/olympus-hermes
$ cd olympus-hermes/hermes
$ docker build -t hermes:latest .
$ cd docker
$ docker-compose up -d
```

Before execute `docker-compose up -d` command it is **strongly recommended** to check the config file!

After these commands a container, called hermes_test, is started. Logs can be checked by `docker logs hermes_test` command.
Execute CLI commands can be done via docker command, some example:
```
$ docker exec -it hermes_test hermes cli -H http://127.0.0.1:3031 list-keys -k /root
/root/status/server1
/root/status/server2
$ docker exec -it hermes_test hermes cli -H http://127.0.0.1:3031 get -k /root/status/server2
offline
```

You can check its log via `docker logs hermes_test` command.

### On host OS

**Note:** This was done on Debian. On other distribution another or different steps might be required.

Another option is to download Hermes, build on your own machine and run there without docker. For this `protobuf-compiler` package must be installed!
If package is installed then by following steps, program can be build and run:
```
$ git clone https://git.thinkaboutit.tech/PublicProjects/olympus-hermes
$ cd olympus-hermes/hermes
$ cargo update
$ cargo build --release
$ export HERMES_LOG=info
$ ./target/release/hermes server -c ./config.toml
2023-06-10T12:08:54.866528Z  INFO hermes::server::utilities::config_parse: Config settings:
2023-06-10T12:08:54.866551Z  INFO hermes::server::utilities::config_parse: - network.classic: Some("127.0.0.1:3030")
2023-06-10T12:08:54.866564Z  INFO hermes::server::utilities::config_parse: - network.grpc: Some("127.0.0.1:3031")
2023-06-10T12:08:54.866571Z  INFO hermes::server::utilities::config_parse: - network.rest: Some("127.0.0.1:3032")
2023-06-10T12:08:54.866576Z  INFO hermes::server::utilities::config_parse: - initials.path: /home/ati/work/OnlyAti.Hermes/hermes/init_data.toml
2023-06-10T12:08:54.866581Z  INFO hermes::server::utilities::config_parse: - logger.location: /tmp/hermes-datastore-test.txt
2023-06-10T12:08:54.866586Z  INFO hermes::server::utilities::config_parse: - scripts.lib_path: ./lua-examples/libs
2023-06-10T12:08:54.866595Z  INFO hermes::server::utilities::config_parse: - scripts.exec_path: ./lua-examples
2023-06-10T12:08:54.868481Z  INFO hermes::server::interfaces: Defined interfaces
2023-06-10T12:08:54.868500Z  INFO hermes::server::interfaces: - HookManager
2023-06-10T12:08:54.868508Z  INFO hermes::server::interfaces: - Datastore
2023-06-10T12:08:54.868522Z  INFO hermes::server::interfaces: - Logger
2023-06-10T12:08:54.868526Z  INFO hermes::server::interfaces: - Classic
2023-06-10T12:08:54.868575Z  INFO hermes::server::interfaces: - gRPC
2023-06-10T12:08:54.868647Z  INFO hermes::server::interfaces: - REST
2023-06-10T12:08:54.869125Z  INFO hermes::server::interfaces::grpc::utilities: gRPC interface on 127.0.0.1:3031 is starting...
2023-06-10T12:08:54.869125Z  INFO hermes::server::interfaces::classic::utilities: classic interface on 127.0.0.1:3030 is starting...
2023-06-10T12:08:54.869886Z  INFO hermes::server::interfaces::rest::utilities: REST interface on 127.0.0.1:3032 is starting...
2023-06-10T12:08:55.870113Z  INFO hermes::server::interfaces: HookManager' is running!
2023-06-10T12:08:55.870142Z  INFO hermes::server::interfaces: Datastore' is running!
2023-06-10T12:08:55.870150Z  INFO hermes::server::interfaces: Logger' is running!
2023-06-10T12:08:55.870157Z  INFO hermes::server::interfaces: Classic' is running!
2023-06-10T12:08:55.870163Z  INFO hermes::server::interfaces: gRPC' is running!
2023-06-10T12:08:55.870174Z  INFO hermes::server::interfaces: REST' is running!
```

Before execute `./target/release/hermes server -c ./config.toml` command it is **strongly recommended** to check the config file!

When it is done, that can take more minutes, in the `target/release/hermes` binary file can be executed. 
To start the server binary must execute like: `hermes server -c /path/to/config.toml`.
Then same binary can be used for CLI thing in form of `hermes cli --help`.

### Via SystemD

Hermes must be downloaded and compiled as it is written [here](README.md#on-host-os)

Run as SystemD service, following steps can be done:
```
$ sudo cp target/release/hermes /usr/local/bin/
$ sudo mkdir -p /usr/var/hermes
$ sudo mkdir -p /usr/var/hermes/lua
$ sudo mkdir -p /usr/var/hermes/lua/lib
$ sudo touch /usr/var/hermes/init.toml
$ sudo mkdir -p /etc/olympus/hermes
$ sudo cp config_default.toml /etc/olympus/hermes/config.toml
```

Create `/etc/systemd/system/hermes.service` file and its content be:
```
[Unit]
Description=Hermes unit in Olympus
After=network.target

[Service]
ExecStart=/usr/local/bin/hermes server -c /etc/olympus/hermes/config.toml
Restart=on-failure
RestartSec=10
Environment="HERMES_LOG=info"

[Install]
WantedBy=multi-user.target
```

When all steps above is done, reload the daemon then start it:
```
$ sudo systemctl daemon-reload
$ sudo systemctl start hermes
```

Logs can be checked by `journalctl -u hermes.service` command.

## Health check

There is a REST API endpoint for health check, this is a `GET /hc`. This can be used, for instance, by cURL to check status:
```
$ curl --fail --connect-timeout 5 127.0.0.1:3032/hc
```
