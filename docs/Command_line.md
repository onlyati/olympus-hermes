# Command line interface

There are two option to work from command line:
1. Using cli option: it is handy to implement into script
1. Using the interactive shell: it is handy for manual work

## CLI mode

CLI is built in the project. It connect to the gRPC interface and using that. The `--help` or `-h` parameter explain what the CLI can do. Overview:
```
$ hermes cli --help
Use Hermes CLI mode

Usage: hermes cli [OPTIONS] --hostname <HOSTNAME> <COMMAND>

Commands:
  get          Get a value of a key
  set          Set value to a key
  rem-key      Remove specified key
  rem-path     Remove path
  list-keys    List keys
  trigger      Send trigger for hooks
  set-hook     Create new hook
  get-hook     Check that a hook exists
  list-hooks   List hooks
  rem-hook     Remove existing hook
  suspend-log  Suspend file writing for database log
  resume-log   Resule file writing for database log
  exec         Execute lua script
  pop          Push value to a queue
  push         Pop value from a queue
  help         Print this message or the help of the given subcommand(s)

Options:
  -H, --hostname <HOSTNAME>  Where it should connect
                             Allowed formats:
                             - <protocol>://<hostname>:<port>, for example ws://127.0.0.1:3043
                             - cfg://<definition-name>, for example: cfg://atihome, it will search  or hostname and CA certificate
  -c, --config <CONFIG>      Config file for connection details [default: /etc/olympus/hermes/client.toml]
  -h, --help                 Print help
```

## Shell mode

Shell can be start by `hermes shell` command. It has a few parameter.

```
$ hermes shell --help
Start a shell to issue CLI commands

Usage: hermes shell [OPTIONS]

Options:
  -H, --hostname <HOSTNAME>  Where it should connect
                             Allowed formats:
                             - <protocol>://<hostname>:<port>, for example ws://127.0.0.1:3043
                             - cfg://<definition-name>, for example: cfg://atihome, it will search  or hostname and CA certificate
  -c, --config <CONFIG>      Config file for connection details [default: /etc/olympus/hermes/client.toml]
  -h, --help                 Print help
```

Once shell is started, it accept the same command then CLI. Next to it, it has other commands that belongs to shell, they are begin with '\' character.
Example usage for the shell:

```
$ hermes shell -c ./client_config.toml
hermes@disconnected=> \?
Hermes shell commands:
\c protocol://host:port   - Connect to a Hermes
\d                        - Disconnect
\l                        - List nodes from client config
\clear                    - Clear screen
\q                        - Quit
hermes@disconnected=> help
Usage: hermes <COMMAND>

Commands:
  get          Get a value of a key
  set          Set value to a key
  rem-key      Remove specified key
  rem-path     Remove path
  list-keys    List keys
  trigger      Send trigger for hooks
  set-hook     Create new hook
  get-hook     Check that a hook exists
  list-hooks   List hooks
  rem-hook     Remove existing hook
  suspend-log  Suspend file writing for database log
  resume-log   Resule file writing for database log
  exec         Execute lua script
  pop          Push value to a queue
  push         Pop value from a queue
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
hermes@disconnected=> \l
dev                 ws://127.0.0.1:3043
test                ws://127.0.0.1:3033
hermes@disconnected=> \c cfg://dev
hermes@ws://127.0.0.1:3043=> list-keys -k /root
/root/ati/hooks/status1
/root/ati/test-script
/root/status/server1
hermes@ws://127.0.0.1:3043=> set -k /root/ati/test -v "This is a some value"
hermes@ws://127.0.0.1:3043=> get -k /root/ati/test
This is a some value
hermes@ws://127.0.0.1:3043=> \d
hermes@disconnected=> \c ws://127.0.0.1:3043
hermes@ws://127.0.0.1:3043=> get -k /root/ati/test
This is a some value
hermes@ws://127.0.0.1:3043=> \q
```