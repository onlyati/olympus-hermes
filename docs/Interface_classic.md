# Classic TCP inteface

These commands does not requires any extra library, they are just simple text that is send over TCP. Default port of classic interface is 3030.

| Command   | Description                        | Example                                           |
|-----------|------------------------------------|---------------------------------------------------|
| SET       | Create or update key               | SET /root/status/server Server is available       |
| GET       | Get value of a key                 | GET /root/status/server                           |
| REMKEY    | Remove specific key                | REMKEY /root/status/server                        |
| REMPATH   | Remove everything under a path     | REMPATH /root/status                              |
| SETHOOK   | Create a new hook                  | SETHOOK /root/status http://127.0.0.1:9999/status |
| GETHOOK   | Get all link for a specific hook   | GETHOOK /root/status                              |
| REMHOOK   | Remove specific hook               | REMHOOK /root/status http://127.0.0.1:9999/status |
| LISTHOOKS | List all hooks under a prefix      | LISTHOOKS /root                                   |
| SUSPEND   | Suspend log                        | SUSPEND LOG                                       |
| RESUME    | Resume log                         | RESUME LOG                                        |

These command can be sent even from bash script by using `socat` utility, for example:
```bash
echo -n "SET /root/status/server Server is available" | socat - tcp:127.0.0.1:3030
```
