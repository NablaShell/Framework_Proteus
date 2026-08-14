// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::config::{StepConfig, Value};
use crate::context::ExecutionContext;
use crate::error::{FrameworkError, Result};
use crate::steps::StepExecutor;

pub struct IfStep;

impl StepExecutor for IfStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let condition = step.parameters.get("condition")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FrameworkError::StepError("Missing 'condition' parameter".to_string()))?;
        
        let interpolated = context.interpolate_string(condition);
        let condition_met = evaluate_condition(&interpolated);
        
        println!("[CONDITION] {} => {}", condition, condition_met);
        
        if let Some(store_as) = &step.store_as {
            context.set_variable(store_as, Value::Boolean(condition_met));
        }
        
        Ok(())
    }
    
    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("condition") {
            return Err(FrameworkError::StepError(
                "If step requires 'condition' parameter".to_string()
            ));
        }
        Ok(())
    }
}

fn evaluate_condition(condition: &str) -> bool {
    let operators = ["==", "!=", ">=", "<=", ">", "<"];
    
    for op in operators {
        if let Some(pos) = condition.find(op) {
            let left = condition[..pos].trim();
            let right = condition[pos + op.len()..].trim();
            
            return match op {
                "==" => left == right,
                "!=" => left != right,
                ">=" => left >= right,
                "<=" => left <= right,
                ">" => left > right,
                "<" => left < right,
                _ => false,
            };
        }
    }
    
    matches!(condition.to_lowercase().as_str(), "true" | "yes" | "1")
}
