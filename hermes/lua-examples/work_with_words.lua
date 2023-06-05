if _G.old ~= nil then
    print("Old key: ", _G.old["key"])
    print("Old value: ", _G.old["value"]) 
else
    print("No old value")
end

print("New key: ", _G.new["key"])
print("New value: ", _G.new["value"])
print("New parm: ", _G.new["parm"])

words = {}
if _G.old ~= nil then
    for word in _G.old["value"]:gmatch("%S+") do
        table.insert(words, word)
    end
end

if _G.new["parm"] == "add" then
    table.insert(words, _G.new["value"])
end

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

result = ""
i = 1
while words[i] ~= nil do
    result = result .. words[i] .. " "
    i = i + 1
end

_G.new["value"] = result
