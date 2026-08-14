-- Пример условной логики
framework.print("Checking conditions...")

-- Получаем переменную из контекста
local value = framework.get_var("threshold")
if value == nil then
    value = 50
    framework.set_var("threshold", value)
end

framework.print("Threshold: " .. tostring(value))

-- Условная логика
if value > 100 then
    framework.set_var("category", "high")
    framework.print("Value is high")
elseif value > 50 then
    framework.set_var("category", "medium")
    framework.print("Value is medium")
else
    framework.set_var("category", "low")
    framework.print("Value is low")
end

-- Циклы
framework.print("Counting to 5:")
for i = 1, 5 do
    framework.print("  Count: " .. i)
    framework.sleep(100)  -- Пауза 100мс
end

-- Функции
local function calculate(a, b)
    return a + b
end

local sum = calculate(10, 20)
framework.set_var("sum", sum)
framework.print("Sum: " .. sum)

-- Таблицы
local config = {
    debug = true,
    max_retries = 3,
    endpoints = {
        primary = "http://localhost:8080",
        backup = "http://localhost:8081"
    }
}

framework.print("Primary endpoint: " .. config.endpoints.primary)
framework.print("Max retries: " .. config.max_retries)

return {
    category = framework.get_var("category"),
    sum = sum,
    config = config
}
