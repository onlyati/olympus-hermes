# Configruration

## Server configuration

There are two configuration: one for the server that is mandatory, another one for cli that is optional. First, let's see the server configuration:

```t
[general]
database_name = "hermes1"     # Name of database, this is the root for each key
logging = true                # Logging into a file to keep persistency or just use in-memory

[network]
classic = "127.0.0.1:3031"     # Classic TCP interface bind to this address
rest = "127.0.0.1:3032"        # REST interface bind to this address
websocket = "127.0.0.1:3033"   # Websocket interface bind to this address

[initials]
# Records and hooks will be read from here during startup
path = "/home/ati/work/OnlyAti.Hermes/hermes/init_data.toml"

[logger]
location = "/tmp/hermes-datastore-test" # Directory for logs

[scripts]
lib_path = "./lua-examples/libs"
exec_path = "./lua-examples"
execs = [
    "test.lua",
    "work_with_words.lua",
    "simple_words.lua",
    "error_example.lua",
]

[gitea]
enable = true
script = "gitea_parser.lua"
key_base = "/hermes1/gitea"
```

**Configuration details**

- General:
  - database_name:
    - This is the identifier of the database
    - Every single key must start with this as root element, for example if its value is 'hermes1' then '/hermes1/test' is a valid key while '/root/test' is not
    - Mandatory field
  - logging:
    - If its value is false, then data is not persistent in database
    - If its value is true, then data might be persistent: persistency is not fully granted as the writes are happen by intervals and events. With other words, just like hermes respond for a SET request it does not mean that it is already written into hermes.af file
    - Mandatory field
- Network:
  - classic: 
    - IP address and port for the TCP socket interface
    - At least one inteface must be specified
  - rest: 
    - IP address and port for the REST interface
    - At least one inteface must be specified
  - websocket: 
    - IP address and port for websocket interface. This interface is available on ws://ip_address:port/ws address.
    - At least one inteface must be specified
- Initials:
  - path:
    - Specify the path for initial file that can contain records and hooks
    - For more details see [Initials](Configuration.md#initials) section
    - Mandatory field
- Logger (mandatory if general.logging is true else optional):
  - location:
    - Directory where Hermes can put its log files
    - If directory does not exist, Hermes try to create it. If failed to create then program is paniciking
- Scripts (optional):
  - Hermes support run stored procedures that can be Lua scripts
  - For more details check [Stored procedures](Stored_procedures.md)
- Gitea (optional):
  - enable:
    - Gitea plugin can be enable/disable
  - script:
    - Name of the Lua script that will be run to process hooks sent by Gitea
    - Script must be available in 'scripts.exec_path' library
  - key_base:
    - Key base that is pass to Gitea Lua script
    - For more details check [Gitea plugin](Gitea_plugin.md)

## Client configuration

Config for cli is optinal only used if cli is called with `cli -H cfg://node1 -c ./client.conf.toml ...` parameter. In this case, node called 'node1' will be looking for in the specified client config. If client config is omitted, default is `/etc/olympus/hermes/client.toml`. See an example for the file, more instance can be defined:

```t
[[node]]
name = "blog"
address = "127.0.0.1:4001"

[[node]]
name = "sandbox"
address = "127.0.0.1:3031"
```

**Configration details**

- Node:
  - name:
    - Name of the node that can be used in shell and cli interface
    - For example if name is 'dev' then cfg://dev can be link to it
  - address:
    - IP address and port number (separated by ':') is specified

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
override = true   # This is even set when value was read from the append file

[[record]]
key = "/root/status/server2"
value = "offline"

```

**Initials details**

- Hook:
  - Specifiy a pre-defined hook
  - prefix:
    - Key prefix the hook, if key is created (SET or PUSH) that begins with this prefix, hook is triggered
  - links:
    - List about links where the hook is sent is form a HTTP POST request
- Record:
  - Specify a pre-defined key-value pair
  - As these keys are created after the hooks are created, these can already trigger the hooks
  - key:
    - Unique key for the value
  - value:
    - Value of the key
  - override:
    - Optional field
    - If logging is enabled, and key is read from append file, then this is not overriden by this value. But when override is set true, then it override the restored value
