# Classic TCP inteface

These commands does not requires any extra library, they are just simple text that is send over TCP. Default port of classic interface is 3030.

| Command   | Description                        | Syntax                                           |
|-----------|------------------------------------|--------------------------------------------------|
| SET       | Create or update key               | SET _key_ _value_                                |
| GET       | Get value of a key                 | GET _key_                                        |
| REMKEY    | Remove specific key                | REMKEY _key_                                     |
| REMPATH   | Remove everything under a path     | REMPATH _key_                                    |
| LIST      | List keys under a path             | LIST _key_                                       |
| TRIGGER   | Trigger hooks                      | TRIGGER _key_ _value_                            |
| SETHOOK   | Create a new hook                  | SETHOOK _prefix_ _link_                          |
| GETHOOK   | Get all link for a specific hook   | GETHOOK _prefix_                                 |
| REMHOOK   | Remove specific hook               | REMHOOK _prefix_ _link_                          |
| LISTHOOKS | List all hooks under a prefix      | LISTHOOKS _prefix_                               |
| SUSPEND   | Suspend log                        | SUSPEND LOG                                      |
| RESUME    | Resume log                         | RESUME LOG                                       |
| EXEC      | Execute lua script                 | EXEC_SET _key_ _script_ _set-or-trigger_ _value_ |

These command can be sent even from bash script by using `socat` utility, for example:
```bash
echo -n "SET /root/status/server Server is available" | socat - tcp:127.0.0.1:3030
```

Fist line of response can be `>Done` of it was successful or `>Error` if command has failed. The further lines are optional, can contains value of the command was a request.
