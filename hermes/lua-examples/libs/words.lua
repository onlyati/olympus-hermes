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
