// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::config::{StepConfig, Value};
use crate::context::ExecutionContext;
use crate::error::{FrameworkError, Result};
use crate::steps::StepExecutor;
use std::process::{Command, Stdio};

/// Шаг для запуска процесса (включая Python)
pub struct SpawnProcessStep;

impl StepExecutor for SpawnProcessStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let command = step.parameters.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FrameworkError::StepError(
                "SpawnProcess step requires 'command' parameter".to_string()
            ))?;
        
        // Интерполируем переменные
        let command = context.interpolate_string(command);
        
        let args = step.parameters.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| context.interpolate_string(s))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        
        let working_dir = step.parameters.get("working_dir")
            .and_then(|v| v.as_str())
            .map(|s| context.interpolate_string(s));
        
        // Проверяем, не Python ли это
        let interpreter = step.parameters.get("interpreter")
            .and_then(|v| v.as_str())
            .map(|s| context.interpolate_string(s));
        
        let mut cmd = if let Some(interp) = interpreter {
            let mut c = Command::new(interp);
            c.arg(&command); // command здесь - это скрипт
            c
        } else if command.ends_with(".py") {
            // Автоматически определяем Python
            let mut c = Command::new("python3");
            c.arg(&command);
            c
        } else {
            Command::new(&command)
        };
        
        cmd.args(&args);
        
        if let Some(dir) = &working_dir {
            cmd.current_dir(dir);
        }
        
        // Настраиваем вывод
        let capture = step.parameters.get("capture_output")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        if capture {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        }
        
        let child = cmd.spawn()
            .map_err(|e| FrameworkError::ProcessError(
                format!("Failed to spawn process '{}': {}", command, e)
            ))?;
        
        let pid = child.id();
        println!("[PROCESS] Started '{}' with PID {}", command, pid);
        
        // Сохраняем PID
        if let Some(store_as) = &step.store_as {
            context.set_variable(store_as, Value::Integer(pid as i64));
        }
        context.set_variable("last_pid", Value::Integer(pid as i64));
        
        // Ожидание завершения
        let wait = step.parameters.get("wait")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if wait {
            let output = child.wait_with_output()
                .map_err(|e| FrameworkError::ProcessError(
                    format!("Failed to wait for process: {}", e)
                ))?;
            
            println!("[PROCESS] Process {} exited with status: {}", pid, output.status);
            
            if let Some(store_output) = step.parameters.get("store_output")
                .and_then(|v| v.as_str()) 
            {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                context.set_variable(store_output, Value::String(stdout));
            }
            
            if let Some(store_error) = step.parameters.get("store_error")
                .and_then(|v| v.as_str()) 
            {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                context.set_variable(store_error, Value::String(stderr));
            }
        }
        
        Ok(())
    }
    
    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("command") {
            return Err(FrameworkError::StepError(
                "SpawnProcess step requires 'command' parameter".to_string()
            ));
        }
        Ok(())
    }
}

/// Шаг для запуска Python скрипта
pub struct RunPythonStep;

impl StepExecutor for RunPythonStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let script = step.parameters.get("script")
            .and_then(|v| v.as_str())
            .map(|s| context.interpolate_string(s));
        
        let script_path = step.parameters.get("script_path")
            .and_then(|v| v.as_str())
            .map(|s| context.interpolate_string(s));
        
        let args = step.parameters.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| context.interpolate_string(s))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        
        // Определяем интерпретатор
        let python = step.parameters.get("python")
            .and_then(|v| v.as_str())
            .unwrap_or("python3");
        
        let mut cmd = Command::new(python);
        
        if let Some(path) = &script_path {
            cmd.arg(path);
        } else if let Some(code) = &script {
            // Используем -c для inline скрипта
            cmd.arg("-c").arg(code);
        } else {
            return Err(FrameworkError::StepError(
                "RunPython step requires 'script' or 'script_path' parameter".to_string()
            ));
        }
        
        cmd.args(&args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        let output = cmd.output()
            .map_err(|e| FrameworkError::ProcessError(
                format!("Failed to run Python script: {}", e)
            ))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        if !stdout.is_empty() {
            println!("[PYTHON] Output:\n{}", stdout);
        }
        
        if !stderr.is_empty() {
            eprintln!("[PYTHON] Error:\n{}", stderr);
        }
        
        if !output.status.success() {
            return Err(FrameworkError::StepError(
                format!("Python script failed with status: {}", output.status)
            ));
        }
        
        // Сохраняем вывод
        if let Some(store_as) = &step.store_as {
            context.set_variable(store_as, Value::String(stdout));
        }
        
        Ok(())
    }
    
    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("script") && !step.parameters.contains_key("script_path") {
            return Err(FrameworkError::StepError(
                "RunPython step requires 'script' or 'script_path' parameter".to_string()
            ));
        }
        Ok(())
    }
}

/// Шаг для остановки процесса
pub struct KillProcessStep;

impl StepExecutor for KillProcessStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let pid = step.parameters.get("pid")
            .and_then(|v| v.as_i64())
            .or_else(|| {
                context.get_variable("last_pid")
                    .ok()
                    .and_then(|v| v.as_i64())
            })
            .ok_or_else(|| FrameworkError::StepError(
                "KillProcess step requires 'pid' parameter or 'last_pid' variable".to_string()
            ))?;
        
        #[cfg(target_os = "linux")]
        {
            unsafe {
                libc::kill(pid as libc::pid_t, libc::SIGTERM);
            }
            println!("[PROCESS] Sent SIGTERM to PID {}", pid);
        }
        
        Ok(())
    }
    
    fn validate(&self, _step: &StepConfig) -> Result<()> {
        Ok(())
    }
}
