# Gitea plugin

Hermes can have enabled endpoint to receive and optionally parse [Gitea hooks](https://docs.gitea.com/features/webhooks). The method is the following:
- Hermes has  a `POST /gitea` endpoint
- Hermes fill `_G.new["key"]` with the specified key prefix and `_G.new["value"]` with the original content of the hook
- Specified script is called from the lua library
- If key or value is empty then it does not save anything, else it saves
- If there is defined hook for the key, then it will be triggered just in case of a regular stored procedure call

## Configuration file

To work with this, configuration file must work correctly. the `[script]` and `[gitea]` part must be defined.
Sample conmfiguration file:
```t
[network]
classic = "0.0.0.0:3031"     # Classic TCP interface bind to this address
rest = "0.0.0.0:3032"        # REST interface bind to this address
websocket = "0.0.0.0:3033"   # REST interface bind to this address

[initials]
# Records and hooks will be read from here during startup
path = "/usr/var/hermes/init.toml"

[logger]
location = "/usr/var/hermes/log.txt" # Which file should the database log written

[scripts]
lib_path = "/usr/var/hermes/lua/libs"  # Additional location where Lua looking for libraries
exec_path = "/usr/var/hermes/lua"      # Hermes looking gitea.script file in this directory
execs = [
    "test.lua",
    "work_with_words.lua",
    "simple_words.lua",
    "error_example.lua",
]

[gitea]
enable = true                 # Turn on/off the feature
script = "gitea_parser.lua"   # Hermes looking for this script in specified exec_path direcotry
key_base = "/root/gitea"      # Default vaalue for _G.new["key"]
```

The `gitea_parser.lua` script must process the data and set the final key. For example:
```lua
json = require "json"

print(_G.new["key"])

value = json.decode(_G.new["value"])
pull_req = value["pull_request"]
url = pull_req["url"]

local t = {}
for str in string.gmatch(url, "([^" .. "/" .. "]+)") do
    table.insert(t, str)
end

_G.new["key"] = _G.new["key"] .. "/" .. t[3] .. "/" .. t[4] .. "/" .. t[5] .. "-" .. t[6]
```
