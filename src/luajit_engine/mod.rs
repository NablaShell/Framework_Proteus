// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

pub mod api;
pub mod registry;

use crate::config::Value;
use crate::context::ExecutionContext;
use crate::error::Result;
use mlua::{Lua, Table, Value as LuaValue};
use std::sync::Arc;
use parking_lot::RwLock;

pub struct LuaJitEngine {
    lua: Lua,
    context: Arc<RwLock<ExecutionContext>>,
}

impl LuaJitEngine {
    pub fn new(context: Arc<RwLock<ExecutionContext>>) -> Result<Self> {
        let lua = Lua::new();
        Ok(Self { lua, context })
    }

    pub fn initialize(&self) -> Result<()> {
        let api_table = self.lua.create_table()?;
        self.lua.globals().set("framework", api_table)?;
        registry::register_all(&self.lua, &self.context)?;
        Ok(())
    }

    pub fn execute_script(&self, script: &str) -> Result<Value> {
        let result: LuaValue = self.lua.load(script).eval()?;
        self.lua_value_to_value(&result)
    }

    pub fn execute_file(&self, path: &str) -> Result<Value> {
        let script = std::fs::read_to_string(path)?;
        self.execute_script(&script)
    }

    pub fn call_function(&self, func_name: &str, args: Vec<Value>) -> Result<Value> {
        let func: mlua::Function = self.lua.globals().get(func_name)?;
        let lua_args: Vec<LuaValue> = args
            .iter()
            .map(|v| self.value_to_lua_value(v))
            .collect::<Result<Vec<_>>>()?;
        let result: LuaValue = func.call(lua_args)?;
        self.lua_value_to_value(&result)
    }

    pub fn get_global(&self, name: &str) -> Result<Value> {
        let value: LuaValue = self.lua.globals().get(name)?;
        self.lua_value_to_value(&value)
    }

    pub fn set_global(&self, name: &str, value: &Value) -> Result<()> {
        let lua_value = self.value_to_lua_value(value)?;
        self.lua.globals().set(name, lua_value)?;
        Ok(())
    }

    fn value_to_lua_value(&self, value: &Value) -> Result<LuaValue<'_>> {
        let lua_value = match value {
            Value::String(s) => LuaValue::String(self.lua.create_string(s)?),
            Value::Integer(i) => LuaValue::Integer(*i),
            Value::Float(f) => LuaValue::Number(*f),
            Value::Boolean(b) => LuaValue::Boolean(*b),
            Value::Array(arr) => {
                let table = self.lua.create_table()?;
                for (idx, item) in arr.iter().enumerate() {
                    let lua_item = self.value_to_lua_value(item)?;
                    table.set(idx + 1, lua_item)?;
                }
                LuaValue::Table(table)
            }
            Value::Object(obj) => {
                let table = self.lua.create_table()?;
                for (key, val) in obj {
                    let lua_val = self.value_to_lua_value(val)?;
                    table.set(key.as_str(), lua_val)?;
                }
                LuaValue::Table(table)
            }
            Value::Null => LuaValue::Nil,
        };
        Ok(lua_value)
    }

    fn lua_value_to_value(&self, value: &LuaValue) -> Result<Value> {
        let value = match value {
            LuaValue::Nil => Value::Null,
            LuaValue::Boolean(b) => Value::Boolean(*b),
            LuaValue::Integer(i) => Value::Integer(*i),
            LuaValue::Number(n) => Value::Float(*n),
            LuaValue::String(s) => Value::String(s.to_str()?.to_string()),
            LuaValue::Table(table) => {
                if self.is_array_table(table) {
                    let mut arr = Vec::new();
                    for i in 1..=table.len()? {
                        let item: LuaValue = table.get(i)?;
                        arr.push(self.lua_value_to_value(&item)?);
                    }
                    Value::Array(arr)
                } else {
                    let mut obj = std::collections::HashMap::new();
                    for pair in table.clone().pairs::<LuaValue, LuaValue>() {
                        let (key, val) = pair?;
                        if let LuaValue::String(k) = key {
                            obj.insert(k.to_str()?.to_string(), self.lua_value_to_value(&val)?);
                        }
                    }
                    Value::Object(obj)
                }
            }
            _ => Value::Null,
        };
        Ok(value)
    }

    fn is_array_table(&self, table: &Table) -> bool {
        let has_one: Option<LuaValue> = table.get(1).ok().flatten();
        let has_zero: Option<LuaValue> = table.get(0).ok().flatten();
        has_one.is_some() && has_zero.is_none()
    }
}
