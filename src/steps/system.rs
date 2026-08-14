// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::config::{StepConfig, Value};
use crate::context::ExecutionContext;
use crate::error::{FrameworkError, Result};
use crate::steps::StepExecutor;
use std::process::Command;

pub struct SystemStep;

impl StepExecutor for SystemStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let command_str = step
            .parameters
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FrameworkError::StepError("Missing 'command' parameter".to_string()))?;

        let command_str = context.interpolate_string(command_str);

        // Проверяем, нужен ли shell для выполнения
        let use_shell = step
            .parameters
            .get("use_shell")
            .and_then(|v| v.as_bool())
            .unwrap_or_else(|| {
                command_str.contains('|')
                    || command_str.contains('>')
                    || command_str.contains('<')
                    || command_str.contains('&')
            });

        let output = if use_shell {
            // Выполняем через shell для поддержки пайпов и редиректов
            #[cfg(target_os = "windows")]
            let shell = "cmd";
            #[cfg(target_os = "windows")]
            let shell_arg = "/C";

            #[cfg(not(target_os = "windows"))]
            let shell = "sh";
            #[cfg(not(target_os = "windows"))]
            let shell_arg = "-c";

            Command::new(shell)
                .arg(shell_arg)
                .arg(&command_str)
                .output()?
        } else {
            // Выполняем напрямую без shell
            let parts: Vec<&str> = command_str.split_whitespace().collect();

            if parts.is_empty() {
                return Err(FrameworkError::StepError("Empty command".to_string()));
            }

            if parts.len() > 1 {
                Command::new(parts[0]).args(&parts[1..]).output()?
            } else {
                Command::new(parts[0]).output()?
            }
        };

        let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Обрезаем вывод если указан max_lines
        if let Some(max_lines) = step.parameters.get("max_lines").and_then(|v| v.as_i64()) {
            let lines: Vec<&str> = stdout.lines().collect();
            let take_count = std::cmp::min(max_lines as usize, lines.len());
            stdout = lines[..take_count].join("\n") + "\n";
        }

        println!("[SYSTEM] Command: {}", command_str);
        if !stdout.is_empty() {
            println!("[SYSTEM] Output: {}", stdout.trim());
        }
        if !stderr.is_empty() {
            eprintln!("[SYSTEM] Error: {}", stderr.trim());
        }

        if let Some(store_as) = &step.store_as {
            let result = if output.status.success() {
                Value::String(stdout)
            } else {
                Value::String(format!("Error: {}", stderr))
            };
            context.set_variable(store_as, result);
        }

        let ignore_errors = step
            .parameters
            .get("ignore_errors")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !output.status.success() && !ignore_errors {
            return Err(FrameworkError::StepError(format!(
                "Command failed with status: {}",
                output.status
            )));
        }

        Ok(())
    }

    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("command") {
            return Err(FrameworkError::StepError(
                "System step requires 'command' parameter".to_string(),
            ));
        }
        Ok(())
    }
}
