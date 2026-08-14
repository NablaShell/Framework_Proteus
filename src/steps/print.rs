// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::config::{StepConfig, Value};
use crate::context::ExecutionContext;
use crate::error::{FrameworkError, Result};
use crate::steps::StepExecutor;

pub struct PrintStep;

impl StepExecutor for PrintStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let message = step
            .parameters
            .get("message")
            .ok_or_else(|| FrameworkError::StepError("Missing 'message' parameter".to_string()))?;

        // Используем рекурсивную интерполяцию
        let interpolated = interpolate_recursive(&message.to_string_value(), context, 0);
        println!("[PRINT] {}", interpolated);

        if let Some(store_as) = &step.store_as {
            context.set_variable(store_as, Value::String(interpolated));
        }

        Ok(())
    }

    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("message") {
            return Err(FrameworkError::StepError(
                "Print step requires 'message' parameter".to_string(),
            ));
        }
        Ok(())
    }
}

// Рекурсивная интерполяция
fn interpolate_recursive(input: &str, context: &ExecutionContext, depth: usize) -> String {
    if depth > 10 {
        return input.to_string();
    }

    let mut result = input.to_string();
    let mut changed = true;

    while changed {
        changed = false;

        for (key, value) in context.get_all_variables() {
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
