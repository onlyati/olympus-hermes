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
