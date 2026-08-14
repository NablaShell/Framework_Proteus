-- Простой пример скрипта
framework.print("Hello from LuaJIT!")

-- Работа с переменными
framework.set_var("lua_message", "This is from Lua")
local message = framework.get_var("lua_message")
framework.print("Got message: " .. message)

-- Логирование
framework.log("info", "This is an info message")
framework.log("warn", "This is a warning")
framework.log("error", "This is an error")

-- Работа со строками
local parts = framework.split("hello,world,test", ",")
framework.print("First part: " .. parts[1])
framework.print("Second part: " .. parts[2])

local joined = framework.join({"a", "b", "c"}, "-")
framework.print("Joined: " .. joined)

-- Таймстамп
local timestamp = framework.get_timestamp()
framework.print("Current time: " .. timestamp)

-- Интерполяция
framework.set_var("name", "World")
local interpolated = framework.interpolate("Hello, ${name}!")
framework.print(interpolated)

return "Script completed successfully"
