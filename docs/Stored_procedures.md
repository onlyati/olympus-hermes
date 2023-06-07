# Stored procedures

Hermes support to run Lua scripts before set or trigger actions. Lua scripts receives the following properties:
- `_G.old["key"]` and `_G.old["key"]`: 
  - If specified key was already existed, then its value is set as a global table
  - If it is a new entry then `_G.old` is `nil`
- `_G.new["key"]` and `_G.new["key"]`: 
  - Key-value pair that was specified in the request, it is never `nil`
- `_G.new["parms"]`:
  - Parameter is an optional field and if it is specified it passed to Lua script
  - If not specified then `_G.new["parms"]` is a `nil`
  - Providing parameter is only available via REST and gRPC interface!

## Content

- [Example for usage](Stored_procedures.md#example-for-usage)
  - [Reason for usage](Stored_procedures.md#reason-for-usage)
  - [Setup Hermes](Stored_procedures.md#setup-hermes)
  - [Test the script](Stored_procedures.md#test-the-script)
  - [How to use custom library](Stored_procedures.md#how-to-use-custom-library)
- [Lua errors](Stored_procedures.md#lua-errors)

## Example for usage

### Reason for usage

Imagine the following scenario:
- There is an API that using Hermes as cache
- This API has 2 running instances
- API receives ticket numbers and it save it as a list into one record

This would be the normal sequence:
1. API#1 check value of /root/ticket/open
1. It is empty, so API#1 create a new pair with "214" vclue
1. API#2 check value of /root/ticket/open and receive "214"
1. API#2 add new ticket and save value, so new value if "214 216"

Although, Hermes update only one pair at the time, but sequence of incoming connections cannot be not determernined.
Due to GET and SET are two different calls, then following situation may happen:
1. API#1 check value of /root/ticket/open and receive "214 216"
1. API#2 check value of /root/ticket/open and receive "214 216"
1. API#1 add new ticket and save value, so new value if "214 216 217"
1. API#1 add new ticket and save value, so new value if "214 216 218"

At the end, one value ("217") has been lost.

### Setup Hermes

In the `config.toml` file, there are the following lines:
```t
[scripts]
lib_path = "/usr/var/hermes/lua/libs"
exec_path = "/usr/var/hermes/lua"
execs = [
    "test.lua",
    "work_with_words.lua",
    "simple_words.lua",
    "error_example.lua",
]

```

Meaning of configuration:
1. `/usr/var/hermes/lua/libs` is set as LUA_PATH environment variable within Hermes
1. Hermes looking for scripts in `/usr/var/hermes/lua/libs` directory
1. Parameter, called `execs`, string list tell which scripts can be called. This list cannot be modified without Hermes restart.

Following script is created as `/usr/var/hermes/lua/work_with_words.lua`. Not requires to be an executable file.
```lua
-- Do not modify if parm is empty
if _G.new["parm"] == nil then
    _G.new["value"] = _G.old["value"]
    return
end

-- Split value for words
words = {}
if _G.old ~= nil then
    for word in _G.old["value"]:gmatch("%S+") do
        table.insert(words, word)
    end
end

-- If it is add then add
if _G.new["parm"] == "add" then
    found = false
    i = 1
    while words[i] ~= nil do
        if words[i] == _G.new["value"] then
            found = true
            break
        end
        i = i + 1
    end
    if found == false then
        table.insert(words, _G.new["value"])
    end
end

-- If it is a remove, then find the word then remove
if _G.new["parm"] == "remove" then
    if _G.old == nil then
        _G.new["value"] = ""
        return
    end
    i = 1
    while words[i] ~= nil do
        if words[i] == _G.new["value"] then
            table.remove(words, i)
            break
        end
        i = i + 1
    end
end

-- Concat the split words into one value again
result = ""
i = 1
while words[i] ~= nil do
    result = result .. words[i] .. " "
    i = i + 1
end

-- At the end, the _G.new["key"] and _G.new["value"] will be saved into Hermes
_G.new["value"] = result
```

### Test the script

Then we can call it, for example via CLI:
```bash
# There is no --save at the end, so it is just a trigger
$ cli -H http://127.0.0.1:3031 exec -k /root/ticket/open -v 124 --script work_with_words.lua --parms add
$ cli -H http://127.0.0.1:3031 get -k /root/ticket/open
Failed request: Invalid key: Specified key does not exist

# Add some ticket
$ cli -H http://127.0.0.1:3031 exec -k /root/ticket/open -v 124 --script work_with_words.lua --parms add --save
$ cli -H http://127.0.0.1:3031 get -k /root/ticket/open
124
$ cli -H http://127.0.0.1:3031 exec -k /root/ticket/open -v 126 --script work_with_words.lua --parms add --save
$ cli -H http://127.0.0.1:3031 get -k /root/ticket/open
124 126
$ cli -H http://127.0.0.1:3031 exec -k /root/ticket/open -v 318 --script work_with_words.lua --parms add --save
$ cli -H http://127.0.0.1:3031 get -k /root/ticket/open
124 126 318

# One has been resolved, delete it
$ cli -H http://127.0.0.1:3031 exec -k /root/ticket/open -v 126 --script work_with_words.lua --parms remove --save
$ cli -H http://127.0.0.1:3031 get -k /root/ticket/open
124 318
```

For more way to call it, check the documentation of interfaces.

### How to use custom library

In `config.toml` file `lib_path` parameter can be specified. If it is specified, then LUA_PATH environment variable is created withion Hermes. 
If the specified path would be `/usr/var/hermes/lua/libs` then value of LUA_PATH is `/usr/var/hermes/lua/libs/?.lua;;`

Same example than above, but using separate file. Create `/usr/var/hermes/lua/libs/words.lua` which content is:
```lua
local word = { version = "0.1" }

function word.split(text)
    words = {}
    for word in text:gmatch("%S+") do
        table.insert(words, word)
    end
    return words
end

function word.found(words, word)
    i = 1
    while words[i] ~= nil do
        if words[i] == word then
            return i
        end
        i = i + 1
    end
    return 0
end

return word
```

Create `/usr/var/hermes/lua/simple_words.lua` file:
```lua
word = require "words"

-- Do not modify if parm is empty
if _G.new["parm"] == nil then
    _G.new["value"] = _G.old["value"]
    return
end

-- Split value for words
words = {}
if _G.old ~= nil then
    words = word.split(_G.old["value"])
end

-- If it is add then add
if _G.new["parm"] == "add" then
    found = word.found(words, _G.new["value"])
    if found == 0 then
        table.insert(words, _G.new["value"])
    end
end

-- If it is a remove, then find the word then remove
if _G.new["parm"] == "remove" then
    if _G.old == nil then
        _G.new["value"] = ""
        return
    end
    i = word.found(words, _G.new["value"])
    if i > 0 then
        table.remove(words, i)
    end
end

-- Concat the split words into one value again
result = ""
i = 1
while words[i] ~= nil do
    result = result .. words[i] .. " "
    i = i + 1
end

-- At the end, the _G.new["key"] and _G.new["value"] will be saved into Hermes
_G.new["value"] = result
```

## Lua errors

If a script fail to run, then error message appear in Hermes log. Let content `/usr/var/hermes/lua/error_example.lua` is:
```lua
-- Generate an error due to dumma does not exist
asd = require "dumma"
```

And call it:
```bash
$ cli -H http://127.0.0.1:3031 exec -k /root/ticket/open -v 133 --script error_example.lua
Failed request: Internal server error
```

Error message from Hermes log:
```
2023-06-07T21:58:41.602036Z ERROR hermes::utilities::lua: failed to execute error_example.lua script
2023-06-07T21:58:41.602127Z ERROR hermes::interfaces::grpc::utilities: error during script exection: runtime error: ./lua-examples/error_example.lua:2: module 'dumma' not found:
2023-06-07T21:58:41.602152Z ERROR hermes::interfaces::grpc::utilities:  no field package.preload['dumma']
2023-06-07T21:58:41.602170Z ERROR hermes::interfaces::grpc::utilities:  no file './lua-examples/libs/dumma.lua'
2023-06-07T21:58:41.602188Z ERROR hermes::interfaces::grpc::utilities:  no file '/usr/local/share/lua/5.4/dumma.lua'
2023-06-07T21:58:41.602207Z ERROR hermes::interfaces::grpc::utilities:  no file '/usr/local/share/lua/5.4/dumma/init.lua'
2023-06-07T21:58:41.602225Z ERROR hermes::interfaces::grpc::utilities:  no file '/usr/local/lib/lua/5.4/dumma.lua'
2023-06-07T21:58:41.602243Z ERROR hermes::interfaces::grpc::utilities:  no file '/usr/local/lib/lua/5.4/dumma/init.lua'
2023-06-07T21:58:41.602261Z ERROR hermes::interfaces::grpc::utilities:  no file './dumma.lua'
2023-06-07T21:58:41.602279Z ERROR hermes::interfaces::grpc::utilities:  no file './dumma/init.lua'
2023-06-07T21:58:41.602297Z ERROR hermes::interfaces::grpc::utilities:
2023-06-07T21:58:41.602315Z ERROR hermes::interfaces::grpc::utilities:  can't load C modules in safe mode
2023-06-07T21:58:41.602333Z ERROR hermes::interfaces::grpc::utilities: stack traceback:
2023-06-07T21:58:41.602350Z ERROR hermes::interfaces::grpc::utilities:  [C]: in ?
2023-06-07T21:58:41.602367Z ERROR hermes::interfaces::grpc::utilities:  [C]: in function 'require'
2023-06-07T21:58:41.602385Z ERROR hermes::interfaces::grpc::utilities:  ./lua-examples/error_example.lua:2: in main chunk
```
