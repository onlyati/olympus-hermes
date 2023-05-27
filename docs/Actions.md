# Hermes actions

## Key-Value pair related actions

This section discusses what actions can be done with the database data itself:
- **SET**: Create a new key-value pair or update existing one. If a pair is set that has a prefix among hooks, then defined hooks are executed.
- **GET**: Get value of a key
- **REMKEY**: Remove specific key
- **REMPATH**: Remove multiple key under a prefix. For example, having a database that has the following keys: `/root/status/server1`, `/root/status/server2`, `/root/status/server3`, `/root/ticket/341`, `/root/ticket/347`. If REMPATH is execute against `/root/status` then all the three key will disappear that begin with this path.

## Hook manager related actions

Hermes has a built-in hook manager, where prefixes can be set and if any key is created or updated within this path, then POST request is sent to the defined addresses. POST request body contain a JSON that contains the key and value:
```json
{
    "key" : "update-or-created-key",
    "value" : "<new-value-of-the-key>"
}
```

Hook manager related actions:
- **TRIGGER**: Hook manager will test the key and send hook if match with predefined prefix. Same effect than with SET but in this case key-value data is not saved
- **SETHOOK**: Create a new hook
- **GETHOOK**: Get links for a specific prefix
- **REMHOOK**: Remove a hook
- **LISTHOOK**: List all hook prefix under a specified path

## Logger actions

Hermes writes an asyncron log about its actions. Its path is defined configuration. It is possible to send a request to Hermes to suspend the logging. It means that Hermes release the logger file and will keep every logging message in memory. Once logging is resumed, Hermes writes the buffered messages and write the file again after every action. This can be useful to arhcive the log without stopping Hermes.

Logger related actions:
- **SUSPEND**: Suspend the log
- **RESUME**: Resume the log
