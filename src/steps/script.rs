// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use crate::config::{StepConfig, Value};
use crate::context::ExecutionContext;
use crate::error::{FrameworkError, Result};
use crate::luajit_engine::LuaJitEngine;
use crate::steps::StepExecutor;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct ScriptStep;

impl StepExecutor for ScriptStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        // Получаем параметры
        let script = step.parameters.get("script")
            .and_then(|v| v.as_str());
        
        let code = step.parameters.get("code")
            .and_then(|v| v.as_str());
        
        let script_path = step.parameters.get("script_path")
            .and_then(|v| v.as_str());
        
        let function = step.parameters.get("function")
            .and_then(|v| v.as_str());
        
        // Создаем Arc для контекста с клонированием данных
        let context_clone = context.clone();
        let context_arc = Arc::new(RwLock::new(context_clone));
        
        // Создаем движок LuaJIT
        let engine = LuaJitEngine::new(Arc::clone(&context_arc))?;
        engine.initialize()?;
        
        // Выполняем скрипт
        let result = if let Some(path) = script_path {
            // Загружаем из файла
            let script_content = std::fs::read_to_string(path)?;
            engine.execute_script(&script_content)?
        } else if let Some(func_name) = function {
            // Вызываем функцию
            let args = extract_args(step)?;
            engine.call_function(func_name, args)?
        } else if let Some(script_content) = script.or(code) {
            // Выполняем inline скрипт
            engine.execute_script(script_content)?
        } else {
            return Err(FrameworkError::StepError(
                "Script step requires 'script', 'code', 'script_path', or 'function' parameter".to_string()
            ));
        };
        
        // Обновляем контекст из Arc
        // Пробуем развернуть Arc, если это возможно
        match Arc::try_unwrap(context_arc) {
            Ok(updated_context) => {
                // Успешно развернули Arc
                let updated_context = updated_context.into_inner();
                *context = updated_context;
            }
            Err(arc) => {
                // Не удалось развернуть, копируем данные
                let updated_context = arc.read().clone();
                *context = updated_context;
            }
        }
        
        // Сохраняем результат ПОСЛЕ обновления контекста
        if let Some(store_as) = &step.store_as {
            context.set_variable(store_as, result);
        }
        
        println!("[SCRIPT] Executed successfully");
        
        Ok(())
    }
    
    fn validate(&self, step: &StepConfig) -> Result<()> {
        let has_script = step.parameters.contains_key("script") || 
                         step.parameters.contains_key("code");
        let has_script_path = step.parameters.contains_key("script_path");
        let has_function = step.parameters.contains_key("function");
        
        if !has_script && !has_script_path && !has_function {
            return Err(FrameworkError::StepError(
                "Script step requires 'script', 'code', 'script_path', or 'function' parameter".to_string()
            ));
        }
        
        Ok(())
    }
}

fn extract_args(step: &StepConfig) -> Result<Vec<Value>> {
    let mut args = Vec::new();
    
    if let Some(args_value) = step.parameters.get("args") {
        if let Value::Array(arr) = args_value {
            args = arr.clone();
        } else {
            args.push(args_value.clone());
        }
    }
    
    Ok(args)
}
