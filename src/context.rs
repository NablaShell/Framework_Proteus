// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use crate::config::Value;
use crate::error::{FrameworkError, Result};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    variables: HashMap<String, Value>,
    step_results: HashMap<String, StepResult>,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub success: bool,
    pub output: Option<Value>,
    pub duration: std::time::Duration,
    pub message: Option<String>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            step_results: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get_variable(&self, name: &str) -> Result<&Value> {
        self.variables
            .get(name)
            .ok_or_else(|| FrameworkError::ContextError(format!("Variable '{}' not found", name)))
    }

    pub fn set_step_result(&mut self, step_id: &str, result: StepResult) {
        self.step_results.insert(step_id.to_string(), result);
    }

    pub fn get_step_result(&self, step_id: &str) -> Option<&StepResult> {
        self.step_results.get(step_id)
    }

    pub fn interpolate_string(&self, input: &str) -> String {
        self.interpolate_recursive(input, 0)
    }

    fn interpolate_recursive(&self, input: &str, depth: usize) -> String {
        if depth > 10 {
            return input.to_string();
        }

        let mut result = input.to_string();
        let mut changed = true;

        while changed {
            changed = false;

            for (key, value) in &self.variables {
                let pattern = format!("${{{}}}", key);
                if result.contains(&pattern) {
                    result = result.replace(&pattern, &value.to_string_value());
                    changed = true;
                }
            }

            if !result.contains("${") {
                break;
            }
        }

        result
    }

    pub fn get_all_variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }
}
