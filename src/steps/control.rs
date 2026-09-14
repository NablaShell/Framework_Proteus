// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use crate::config::StepConfig; // Убрали Value
use crate::context::ExecutionContext;
use crate::error::{FrameworkError, Result};
use crate::steps::StepExecutor;
use std::thread;
use std::time::Duration;

pub struct SleepStep;

impl StepExecutor for SleepStep {
    fn execute(&self, step: &StepConfig, _context: &mut ExecutionContext) -> Result<()> {
        let duration_ms = step
            .parameters
            .get("duration_ms")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| {
                FrameworkError::StepError("Missing 'duration_ms' parameter".to_string())
            })?;

        if duration_ms < 0 {
            return Err(FrameworkError::StepError(
                "Duration must be non-negative".to_string(),
            ));
        }

        println!("[CONTROL] Sleeping for {} ms", duration_ms);
        thread::sleep(Duration::from_millis(duration_ms as u64));

        Ok(())
    }

    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("duration_ms") {
            return Err(FrameworkError::StepError(
                "Sleep step requires 'duration_ms' parameter".to_string(),
            ));
        }
        Ok(())
    }
}

pub struct LogStep;

impl StepExecutor for LogStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let level = step
            .parameters
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info");

        let message = step
            .parameters
            .get("message")
            .ok_or_else(|| FrameworkError::StepError("Missing 'message' parameter".to_string()))?;

        let interpolated = context.interpolate_string(&message.to_string_value());

        match level {
            "debug" => log::debug!("{}", interpolated),
            "info" => log::info!("{}", interpolated),
            "warn" => log::warn!("{}", interpolated),
            "error" => log::error!("{}", interpolated),
            _ => println!("[LOG:{}] {}", level, interpolated),
        }

        Ok(())
    }

    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("message") {
            return Err(FrameworkError::StepError(
                "Log step requires 'message' parameter".to_string(),
            ));
        }
        Ok(())
    }
}
