# Command line (CLI) commands

CLI is built in the project. It connect to the gRPC interface and using that. The `--help` or `-h` parameter explain what the CLI can do. Overview:
```
cli --help
Usage: cli [OPTIONS] --hostname <HOSTNAME> <COMMAND>

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
  help         Print this message or the help of the given subcommand(s)

Options:
  -H, --hostname <HOSTNAME>  Where it should connect
                             Allowed formats:
                             - <protocol>://<hostname>:<port>, for example http://127.0.0.1:3041
                             - cfg://<definition-name>, for example: cfg://atihome, it will search  or hostname and CA certificate
  -c, --config <CONFIG>      Config file for connection details [default: /etc/olympus/hermes/client.toml]
  -h, --help                 Print help
```

