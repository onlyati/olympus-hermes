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
