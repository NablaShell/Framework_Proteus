// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::config::Value;
use crate::context::ExecutionContext;
use crate::error::Result;
use mlua::{Lua, Table, Value as LuaValue};
use std::sync::Arc;
use parking_lot::RwLock;

pub fn register_all(lua: &Lua, context: &Arc<RwLock<ExecutionContext>>) -> Result<()> {
    // Создаем основную таблицу API
    let framework = lua.create_table()?;
    
    // Регистрируем базовые функции
    register_variables_api(lua, &framework, context)?;
    register_logging_api(lua, &framework)?;
    register_system_api(lua, &framework)?;
    register_string_api(lua, &framework)?;
    register_json_api(lua, &framework)?;
    register_context_api(lua, &framework, context)?;
    
    // Устанавливаем глобальную таблицу framework
    lua.globals().set("framework", framework)?;
    
    // Устанавливаем глобальные функции для удобства
    set_global_functions(lua, context)?;
    
    Ok(())
}

fn register_variables_api(
    lua: &Lua,
    table: &Table,
    context: &Arc<RwLock<ExecutionContext>>,
) -> Result<()> {
    // framework.set_var(name, value)
    let ctx = Arc::clone(context);
    let set_var = lua.create_function(move |_, (name, value): (String, LuaValue)| {
        let mut ctx = ctx.write(); // Изменено на write()
        let value = convert_lua_to_value(&value)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        ctx.set_variable(&name, value);
        Ok(())
    })?;
    table.set("set_var", set_var)?;
    
    // framework.get_var(name) -> value
    let ctx = Arc::clone(context);
    let get_var = lua.create_function(move |lua, name: String| {
        let ctx = ctx.read();
        match ctx.get_variable(&name) {
            Ok(value) => convert_value_to_lua(lua, value)
                .map_err(|e| mlua::Error::RuntimeError(e.to_string())),
            Err(_) => Ok(LuaValue::Nil),
        }
    })?;
    table.set("get_var", get_var)?;
    
    // framework.set_variable(name, value) - алиас
    let ctx = Arc::clone(context);
    let set_variable = lua.create_function(move |_, (name, value): (String, LuaValue)| {
        let mut ctx = ctx.write(); // Изменено на write()
        let value = convert_lua_to_value(&value)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        ctx.set_variable(&name, value);
        Ok(())
    })?;
    table.set("set_variable", set_variable)?;
    
    // framework.get_variable(name) -> value - алиас
    let ctx = Arc::clone(context);
    let get_variable = lua.create_function(move |lua, name: String| {
        let ctx = ctx.read();
        match ctx.get_variable(&name) {
            Ok(value) => convert_value_to_lua(lua, value)
                .map_err(|e| mlua::Error::RuntimeError(e.to_string())),
            Err(_) => Ok(LuaValue::Nil),
        }
    })?;
    table.set("get_variable", get_variable)?;
    
    // framework.has_variable(name) -> bool
    let ctx = Arc::clone(context);
    let has_variable = lua.create_function(move |_, name: String| {
        let ctx = ctx.read();
        Ok(ctx.get_variable(&name).is_ok())
    })?;
    table.set("has_variable", has_variable)?;
    
    Ok(())
}

fn register_logging_api(lua: &Lua, table: &Table) -> Result<()> {
    // framework.log(level, message)
    let log_func = lua.create_function(|_, (level, message): (String, String)| {
        match level.as_str() {
            "debug" => log::debug!("[LUA] {}", message),
            "info" => log::info!("[LUA] {}", message),
            "warn" => log::warn!("[LUA] {}", message),
            "error" => log::error!("[LUA] {}", message),
            _ => println!("[LUA:{}] {}", level, message),
        }
        Ok(())
    })?;
    table.set("log", log_func)?;
    
    // framework.print(message)
    let print_func = lua.create_function(|_, message: String| {
        println!("[LUA] {}", message);
        Ok(())
    })?;
    table.set("print", print_func)?;
    
    // framework.info(message)
    let info_func = lua.create_function(|_, message: String| {
        log::info!("[LUA] {}", message);
        Ok(())
    })?;
    table.set("info", info_func)?;
    
    // framework.warn(message)
    let warn_func = lua.create_function(|_, message: String| {
        log::warn!("[LUA] {}", message);
        Ok(())
    })?;
    table.set("warn", warn_func)?;
    
    // framework.error(message)
    let error_func = lua.create_function(|_, message: String| {
        log::error!("[LUA] {}", message);
        Ok(())
    })?;
    table.set("error", error_func)?;
    
    // framework.debug(message)
    let debug_func = lua.create_function(|_, message: String| {
        log::debug!("[LUA] {}", message);
        Ok(())
    })?;
    table.set("debug", debug_func)?;
    
    Ok(())
}

fn register_system_api(lua: &Lua, table: &Table) -> Result<()> {
    // framework.exec(command) -> table
    let exec_func = lua.create_function(|lua, command: String| {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        
        let result = lua.create_table()?;
        result.set("success", output.status.success())?;
        result.set("stdout", String::from_utf8_lossy(&output.stdout).to_string())?;
        result.set("stderr", String::from_utf8_lossy(&output.stderr).to_string())?;
        result.set("code", output.status.code().unwrap_or(-1))?;
        
        Ok(result)
    })?;
    table.set("exec", exec_func)?;
    
    // framework.execute_command(command) -> table - алиас
    let execute_command = lua.create_function(|lua, command: String| {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        
        let result = lua.create_table()?;
        result.set("success", output.status.success())?;
        result.set("stdout", String::from_utf8_lossy(&output.stdout).to_string())?;
        result.set("stderr", String::from_utf8_lossy(&output.stderr).to_string())?;
        result.set("code", output.status.code().unwrap_or(-1))?;
        
        Ok(result)
    })?;
    table.set("execute_command", execute_command)?;
    
    // framework.sleep(ms)
    let sleep_func = lua.create_function(|_, duration_ms: u64| {
        std::thread::sleep(std::time::Duration::from_millis(duration_ms));
        Ok(())
    })?;
    table.set("sleep", sleep_func)?;
    
    Ok(())
}

fn register_string_api(lua: &Lua, table: &Table) -> Result<()> {
    // framework.split(str, delim) -> array
    let split_func = lua.create_function(|lua, (input, delim): (String, String)| {
        let parts: Vec<&str> = input.split(&delim).collect();
        let result = lua.create_table()?;
        for (i, part) in parts.iter().enumerate() {
            result.set(i + 1, *part)?;
        }
        Ok(result)
    })?;
    table.set("split", split_func)?;
    
    // framework.string_split(str, delim) -> array - алиас
    let string_split = lua.create_function(|lua, (input, delim): (String, String)| {
        let parts: Vec<&str> = input.split(&delim).collect();
        let result = lua.create_table()?;
        for (i, part) in parts.iter().enumerate() {
            result.set(i + 1, *part)?;
        }
        Ok(result)
    })?;
    table.set("string_split", string_split)?;
    
    // framework.join(array, delim) -> string
    let join_func = lua.create_function(|_, (arr, delim): (Table, String)| {
        let mut parts = Vec::new();
        for i in 1..=arr.len()? {
            if let Ok(part) = arr.get::<_, String>(i) {
                parts.push(part);
            }
        }
        Ok(parts.join(&delim))
    })?;
    table.set("join", join_func)?;
    
    // framework.string_join(array, delim) -> string - алиас
    let string_join = lua.create_function(|_, (arr, delim): (Table, String)| {
        let mut parts = Vec::new();
        for i in 1..=arr.len()? {
            if let Ok(part) = arr.get::<_, String>(i) {
                parts.push(part);
            }
        }
        Ok(parts.join(&delim))
    })?;
    table.set("string_join", string_join)?;
    
    Ok(())
}

fn register_json_api(lua: &Lua, table: &Table) -> Result<()> {
    // framework.json_encode(value) -> string
    let encode_func = lua.create_function(|_, value: LuaValue| {
        let value = convert_lua_to_value(&value)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        serde_json::to_string(&value)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
    })?;
    table.set("json_encode", encode_func)?;
    
    // framework.json_decode(str) -> value
    let decode_func = lua.create_function(|lua, json_str: String| {
        let value: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        convert_json_to_lua(lua, &value)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
    })?;
    table.set("json_decode", decode_func)?;
    
    Ok(())
}

fn register_context_api(
    lua: &Lua,
    table: &Table,
    context: &Arc<RwLock<ExecutionContext>>,
) -> Result<()> {
    // framework.interpolate(str) -> string
    let ctx = Arc::clone(context);
    let interpolate = lua.create_function(move |_, input: String| {
        let ctx = ctx.read();
        Ok(ctx.interpolate_string(&input))
    })?;
    table.set("interpolate", interpolate)?;
    
    // framework.get_timestamp() -> string
    let timestamp = lua.create_function(|_, ()| {
        Ok(chrono::Utc::now().to_rfc3339())
    })?;
    table.set("get_timestamp", timestamp)?;
    
    // framework.get_all_variables() -> table
    let ctx = Arc::clone(context);
    let get_all_vars = lua.create_function(move |lua, ()| {
        let ctx = ctx.read();
        let result = lua.create_table()?;
        for (key, value) in ctx.get_all_variables() {
            let lua_value = convert_value_to_lua(lua, value)
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
            result.set(key.as_str(), lua_value)?;
        }
        Ok(result)
    })?;
    table.set("get_all_variables", get_all_vars)?;
    
    Ok(())
}

fn set_global_functions(lua: &Lua, context: &Arc<RwLock<ExecutionContext>>) -> Result<()> {
    // Глобальная функция get_var
    let ctx = Arc::clone(context);
    let get_var = lua.create_function(move |lua, name: String| {
        let ctx = ctx.read();
        match ctx.get_variable(&name) {
            Ok(value) => convert_value_to_lua(lua, value)
                .map_err(|e| mlua::Error::RuntimeError(e.to_string())),
            Err(_) => Ok(LuaValue::Nil),
        }
    })?;
    lua.globals().set("get_var", get_var)?;
    
    // Глобальная функция set_var
    let ctx = Arc::clone(context);
    let set_var = lua.create_function(move |_, (name, value): (String, LuaValue)| {
        let mut ctx = ctx.write(); // Изменено на write()
        let value = convert_lua_to_value(&value)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        ctx.set_variable(&name, value);
        Ok(())
    })?;
    lua.globals().set("set_var", set_var)?;
    
    Ok(())
}

// Функции конвертации с правильными lifetimes
fn convert_value_to_lua<'lua>(lua: &'lua Lua, value: &Value) -> Result<LuaValue<'lua>> {
    let lua_value = match value {
        Value::String(s) => LuaValue::String(lua.create_string(s)?),
        Value::Integer(i) => LuaValue::Integer(*i),
        Value::Float(f) => LuaValue::Number(*f),
        Value::Boolean(b) => LuaValue::Boolean(*b),
        Value::Array(arr) => {
            let table = lua.create_table()?;
            for (idx, item) in arr.iter().enumerate() {
                let lua_item = convert_value_to_lua(lua, item)?;
                table.set(idx + 1, lua_item)?;
            }
            LuaValue::Table(table)
        }
        Value::Object(obj) => {
            let table = lua.create_table()?;
            for (key, val) in obj {
                let lua_val = convert_value_to_lua(lua, val)?;
                table.set(key.as_str(), lua_val)?;
            }
            LuaValue::Table(table)
        }
        Value::Null => LuaValue::Nil,
    };
    
    Ok(lua_value)
}

fn convert_lua_to_value(value: &LuaValue) -> Result<Value> {
    let value = match value {
        LuaValue::Nil => Value::Null,
        LuaValue::Boolean(b) => Value::Boolean(*b),
        LuaValue::Integer(i) => Value::Integer(*i),
        LuaValue::Number(n) => Value::Float(*n),
        LuaValue::String(s) => Value::String(s.to_str()?.to_string()),
        LuaValue::Table(table) => {
            // Определяем, массив это или объект
            let is_array = table.get::<_, Option<LuaValue>>(1)?.is_some();
            
            if is_array {
                let mut arr = Vec::new();
                for i in 1..=table.len()? {
                    let item: LuaValue = table.get(i)?;
                    arr.push(convert_lua_to_value(&item)?);
                }
                Value::Array(arr)
            } else {
                let mut obj = std::collections::HashMap::new();
                for pair in table.clone().pairs::<String, LuaValue>() {
                    let (key, val) = pair?;
                    obj.insert(key, convert_lua_to_value(&val)?);
                }
                Value::Object(obj)
            }
        }
        _ => Value::Null,
    };
    
    Ok(value)
}

fn convert_json_to_lua<'lua>(lua: &'lua Lua, value: &serde_json::Value) -> Result<LuaValue<'lua>> {
    let lua_value = match value {
        serde_json::Value::Null => LuaValue::Nil,
        serde_json::Value::Bool(b) => LuaValue::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                LuaValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                LuaValue::Number(f)
            } else {
                LuaValue::Nil
            }
        }
        serde_json::Value::String(s) => LuaValue::String(lua.create_string(s)?),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, item) in arr.iter().enumerate() {
                let lua_item = convert_json_to_lua(lua, item)?;
                table.set(i + 1, lua_item)?;
            }
            LuaValue::Table(table)
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table()?;
            for (key, val) in obj {
                let lua_val = convert_json_to_lua(lua, val)?;
                table.set(key.as_str(), lua_val)?;
            }
            LuaValue::Table(table)
        }
    };
    
    Ok(lua_value)
}
