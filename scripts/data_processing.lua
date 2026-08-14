-- Пример обработки данных
framework.print("Starting data processing...")

-- Создаем тестовые данные
local data = {
    name = "Test User",
    age = 30,
    email = "test@example.com",
    tags = {"admin", "user", "active"}
}

-- Работа с JSON
local json_str = framework.json_encode(data)
framework.print("JSON: " .. json_str)

local decoded = framework.json_decode(json_str)
framework.print("Decoded name: " .. decoded.name)
framework.print("Decoded age: " .. tostring(decoded.age))

-- Работа с массивами
framework.print("Tags:")
for i, tag in ipairs(decoded.tags) do
    framework.print("  " .. i .. ": " .. tag)
end

-- Выполнение системной команды
local result = framework.exec("echo 'Command executed from Lua'")
if result.success then
    framework.print("Command output: " .. result.stdout)
else
    framework.print("Command failed: " .. result.stderr)
end

-- Сохраняем результат
framework.set_var("processed_data", json_str)
framework.set_var("user_count", 1)

return {
    success = true,
    message = "Data processed successfully",
    timestamp = framework.get_timestamp()
}
