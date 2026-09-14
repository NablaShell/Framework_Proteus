// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use crate::config::Value;
use crate::context::ExecutionContext;
use crate::error::Result;
use mlua::{Lua, Table, Value as LuaValue};
use std::sync::Arc;
use parking_lot::RwLock;

pub fn create_api_table<'lua>(
    lua: &'lua Lua,
    context: &Arc<RwLock<ExecutionContext>>,
) -> Result<Table<'lua>> {
    let api = lua.create_table()?;

    // === Работа с переменными ===
    
    // set_variable(name, value)
    let set_var_ctx = Arc::clone(context);
    let set_var = lua.create_function(move |_, (name, value): (String, LuaValue)| {
        let mut ctx = set_var_ctx.write(); // Изменено на write()
        let value = lua_value_to_value(&value)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        ctx.set_variable(&name, value);
        Ok(())
    })?;
    api.set("set_variable", set_var)?;
    
    // get_variable(name) -> value
    let get_var_ctx = Arc::clone(context);
    let get_var = lua.create_function(move |lua, name: String| {
        let ctx = get_var_ctx.read();
        match ctx.get_variable(&name) {
            Ok(value) => value_to_lua_value(lua, value)
                .map_err(|e| mlua::Error::RuntimeError(e.to_string())),
            Err(_) => Ok(LuaValue::Nil),
        }
    })?;
    api.set("get_variable", get_var)?;
    
    // has_variable(name) -> bool
    let has_var_ctx = Arc::clone(context);
    let has_var = lua.create_function(move |_, name: String| {
        let ctx = has_var_ctx.read();
        Ok(ctx.get_variable(&name).is_ok())
    })?;
    api.set("has_variable", has_var)?;
    
    // get_all_variables() -> table
    let get_all_ctx = Arc::clone(context);
    let get_all = lua.create_function(move |lua, ()| {
        let ctx = get_all_ctx.read();
        let table = lua.create_table()?;
        for (key, value) in ctx.get_all_variables() {
            let lua_value = value_to_lua_value(lua, value)
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
            table.set(key.as_str(), lua_value)?;
        }
        Ok(table)
    })?;
    api.set("get_all_variables", get_all)?;
    
    // === Логирование ===
    
    // log(level, message)
    let log_func = lua.create_function(|_, (level, message): (String, String)| {
        match level.as_str() {
            "debug" => log::debug!("{}", message),
            "info" => log::info!("{}", message),
            "warn" => log::warn!("{}", message),
            "error" => log::error!("{}", message),
            _ => println!("[LUA:{}] {}", level, message),
        }
        Ok(())
    })?;
    api.set("log", log_func)?;
    
    // print(message)
    let print_func = lua.create_function(|_, message: String| {
        println!("[LUA] {}", message);
        Ok(())
    })?;
    api.set("print", print_func)?;
    
    // === Системные функции ===
    
    // execute_command(command) -> (success, stdout, stderr)
    let exec_func = lua.create_function(|lua, command: String| {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        
        let table = lua.create_table()?;
        table.set("success", output.status.success())?;
        table.set("stdout", String::from_utf8_lossy(&output.stdout).to_string())?;
        table.set("stderr", String::from_utf8_lossy(&output.stderr).to_string())?;
        table.set("exit_code", output.status.code().unwrap_or(-1))?;
        
        Ok(table)
    })?;
    api.set("execute_command", exec_func)?;
    
    // sleep(ms)
    let sleep_func = lua.create_function(|_, duration_ms: u64| {
        std::thread::sleep(std::time::Duration::from_millis(duration_ms));
        Ok(())
    })?;
    api.set("sleep", sleep_func)?;
    
    // === Утилиты ===
    
    // interpolate(string) -> string
    let interp_ctx = Arc::clone(context);
    let interpolate = lua.create_function(move |_, input: String| {
        let ctx = interp_ctx.read();
        Ok(ctx.interpolate_string(&input))
    })?;
    api.set("interpolate", interpolate)?;
    
    // get_timestamp() -> string
    let timestamp = lua.create_function(|_, ()| {
        Ok(chrono::Utc::now().to_rfc3339())
    })?;
    api.set("get_timestamp", timestamp)?;
    
    // === Работа со строками ===
    
    // string_split(string, delimiter) -> array
    let split_func = lua.create_function(|lua, (input, delim): (String, String)| {
        let parts: Vec<&str> = input.split(&delim).collect();
        let table = lua.create_table()?;
        for (i, part) in parts.iter().enumerate() {
            table.set(i + 1, *part)?;
        }
        Ok(table)
    })?;
    api.set("string_split", split_func)?;
    
    // string_join(array, delimiter) -> string
    let join_func = lua.create_function(|_, (arr, delim): (Table, String)| {
        let mut parts = Vec::new();
        for i in 1..=arr.len()? {
            if let Ok(part) = arr.get::<_, String>(i) {
                parts.push(part);
            }
        }
        Ok(parts.join(&delim))
    })?;
    api.set("string_join", join_func)?;
    
    // === Работа с JSON ===
    
    // json_encode(table) -> string
    let json_encode = lua.create_function(|_, value: LuaValue| {
        let value = lua_value_to_value(&value)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        serde_json::to_string(&value)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
    })?;
    api.set("json_encode", json_encode)?;
    
    // json_decode(string) -> table
    let json_decode = lua.create_function(|lua, json_str: String| {
        let value: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        json_value_to_lua(lua, &value)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
    })?;
    api.set("json_decode", json_decode)?;
    
    Ok(api)
}

// Вспомогательные функции конвертации

fn value_to_lua_value<'lua>(lua: &'lua Lua, value: &Value) -> Result<LuaValue<'lua>> {
    let lua_value = match value {
        Value::String(s) => LuaValue::String(lua.create_string(s)?),
        Value::Integer(i) => LuaValue::Integer(*i),
        Value::Float(f) => LuaValue::Number(*f),
        Value::Boolean(b) => LuaValue::Boolean(*b),
        Value::Array(arr) => {
            let table = lua.create_table()?;
            for (idx, item) in arr.iter().enumerate() {
                let lua_item = value_to_lua_value(lua, item)?;
                table.set(idx + 1, lua_item)?;
            }
            LuaValue::Table(table)
        }
        Value::Object(obj) => {
            let table = lua.create_table()?;
            for (key, val) in obj {
                let lua_val = value_to_lua_value(lua, val)?;
                table.set(key.as_str(), lua_val)?;
            }
            LuaValue::Table(table)
        }
        Value::Null => LuaValue::Nil,
    };
    
    Ok(lua_value)
}

fn lua_value_to_value(value: &LuaValue) -> Result<Value> {
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
                    arr.push(lua_value_to_value(&item)?);
                }
                Value::Array(arr)
            } else {
                let mut obj = std::collections::HashMap::new();
                for pair in table.clone().pairs::<String, LuaValue>() {
                    let (key, val) = pair?;
                    obj.insert(key, lua_value_to_value(&val)?);
                }
                Value::Object(obj)
            }
        }
        _ => Value::Null,
    };
    
    Ok(value)
}

fn json_value_to_lua<'lua>(lua: &'lua Lua, value: &serde_json::Value) -> Result<LuaValue<'lua>> {
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
                let lua_item = json_value_to_lua(lua, item)?;
                table.set(i + 1, lua_item)?;
            }
            LuaValue::Table(table)
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table()?;
            for (key, val) in obj {
                let lua_val = json_value_to_lua(lua, val)?;
                table.set(key.as_str(), lua_val)?;
            }
            LuaValue::Table(table)
        }
    };
    
    Ok(lua_value)
}
