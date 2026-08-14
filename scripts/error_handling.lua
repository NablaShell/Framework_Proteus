-- Пример обработки ошибок
framework.print("Testing error handling...")

-- Функция с обработкой ошибок
local function safe_divide(a, b)
    local status, result = pcall(function()
        if b == 0 then
            error("Division by zero")
        end
        return a / b
    end)
    
    if status then
        return result
    else
        framework.log("error", "Error in division: " .. tostring(result))
        return nil
    end
end

-- Тестируем
local result1 = safe_divide(10, 2)
framework.print("10 / 2 = " .. tostring(result1))

local result2 = safe_divide(10, 0)
framework.print("10 / 0 = " .. tostring(result2))

-- Работа с nil значениями
local value = framework.get_var("nonexistent")
if value == nil then
    framework.print("Variable doesn't exist (expected)")
end

-- Проверка типов
local test_value = "123"
if type(test_value) == "string" then
    framework.print("test_value is a string")
    local number = tonumber(test_value)
    framework.print("Converted to number: " .. tostring(number))
end

return {
    success = true,
    division_results = {result1, result2},
    nil_handled = true
}
