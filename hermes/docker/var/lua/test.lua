asd = require "dumma"
data = require "dummy"

str_index = _G.new["value"]
index = tonumber(str_index)
_G.new["value"] = data[index]
