print("From lua key:", _G.key)
print("From lua value:", _G.value)
print("From lua param action:", _G.params["action"])
print("From lua param file:", _G.params["file"])

words = {}
for word in _G.value:gmatch("%S+") do
    table.insert(words, word)
end

if _G.params["action"] == "add" then
    table.insert(words, _G.params["file"])
end

if _G.params["action"] == "remove" then
    i = 1
    while words[i] ~= nil do
        if words[i] == _G.params["file"] then
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

_G.value = result
