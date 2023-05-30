# Configruration

There are two configuration: one for the server that is mandatory, another one for cli that is optional. First, let's see the server configuration:

```t
[network]
classic = "127.0.0.1:3030"     # Classic TCP interface bind to this address
grpc = "127.0.0.1:3031"        # gRPC interface bind to this address
rest = "127.0.0.1:3032"        # REST interface bind to this address

[initials]
# Records and hooks will be read from here during startup
path = "/usr/var/hermes/init.toml"

[logger]
location = "/usr/var/hermes/log.txt" # Which file should the database log written

```

Config for cli is optinal only used if cli is called with `cli -H cfg://node1 -c ./client.conf.toml ...` parameter. In this case, node called 'node1' will be looking for in the specified client config. If client config is omitted, default is `/etc/olympus/hermes/client.toml`. See an example for the file, more instance can be defined:

```t
[[node]]
name = "blog"
address = "127.0.0.1:4001"

[[node]]
name = "sandbox"
address = "127.0.0.1:3031"
```

## Initials

During startup, initial file is read that can contains hooks and records. First hooks are processed, then records. At this point, record is able to already trigger hook. Sample for initial file:

```t
[[hook]]
prefix = "/root/status/hooks"
links = ["http://127.0.0.1:9999/status-update", "http://127.0.0.1:9999/status-update-2"]

[[hook]]
prefix = "/root/agent/hooks"
links = ["http://127.0.0.1:9999/agent-update"]

[[record]]
key = "/root/status/server1"
value = "online"

[[record]]
key = "/root/status/server2"
value = "offline"

```