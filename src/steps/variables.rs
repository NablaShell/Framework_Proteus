// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::config::{StepConfig, Value};
use crate::context::ExecutionContext;
use crate::error::{FrameworkError, Result};
use crate::steps::StepExecutor;

pub struct SetVariableStep;

impl StepExecutor for SetVariableStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let name = step
            .parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FrameworkError::StepError("Missing 'name' parameter".to_string()))?;

        let value = step
            .parameters
            .get("value")
            .ok_or_else(|| FrameworkError::StepError("Missing 'value' parameter".to_string()))?;

        // Интерполяция строковых значений
        let interpolated_value = match value {
            Value::String(s) => {
                let interpolated = context.interpolate_string(s);
                Value::String(interpolated)
            }
            other => other.clone(),
        };

        context.set_variable(name, interpolated_value.clone());

        if let Some(store_as) = &step.store_as {
            context.set_variable(store_as, interpolated_value);
        }

        println!(
            "[VARIABLE] Set {} = {}",
            name,
            context.get_variable(name)?.to_string_value()
        );
        Ok(())
    }

    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("name") || !step.parameters.contains_key("value") {
            return Err(FrameworkError::StepError(
                "SetVariable step requires 'name' and 'value' parameters".to_string(),
            ));
        }
        Ok(())
    }
}

pub struct GetVariableStep;

impl StepExecutor for GetVariableStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let name = step
            .parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FrameworkError::StepError("Missing 'name' parameter".to_string()))?;

        let value = context.get_variable(name)?;
        println!("[VARIABLE] {} = {}", name, value.to_string_value());

        if let Some(store_as) = &step.store_as {
            context.set_variable(store_as, value.clone());
        }

        Ok(())
    }

    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("name") {
            return Err(FrameworkError::StepError(
                "GetVariable step requires 'name' parameter".to_string(),
            ));
        }
        Ok(())
    }
}
