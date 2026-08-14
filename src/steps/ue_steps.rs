// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::config::{StepConfig, Value};
use crate::context::ExecutionContext;
use crate::error::{FrameworkError, Result};
use crate::memory::process::ProcessMemory;
use crate::steps::StepExecutor;
use crate::ue::scanner::UEScanner;
use crate::ue::uobject::UObjectWalker;

/// Шаг для поиска UE структур
pub struct UEScanStep;

impl Default for UEScanStep {
    fn default() -> Self {
        Self
    }
}

impl UEScanStep {
    pub fn new() -> Self {
        Self
    }
}

impl StepExecutor for UEScanStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        // Интерполируем pid из параметров
        let pid = step.parameters.get("pid")
            .and_then(|v| v.as_i64())
            .or_else(|| {
                // Пробуем получить как строку и интерполировать
                step.parameters.get("pid")
                    .and_then(|v| v.as_str())
                    .map(|s| context.interpolate_string(s))
                    .and_then(|s| s.parse::<i64>().ok())
            })
            .or_else(|| {
                context.get_variable("last_pid")
                    .ok()
                    .and_then(|v| v.as_i64())
            })
            .or_else(|| {
                // Пробуем получить из target_pid переменной
                context.get_variable("target_pid")
                    .ok()
                    .and_then(|v| v.as_i64())
            })
            .ok_or_else(|| FrameworkError::StepError(
                "UEScan step requires 'pid' or 'last_pid'".to_string()
            ))?;
        
        let process = ProcessMemory::new(pid as u32)?;
        let scanner = UEScanner::new();
        
        println!("[UE] Scanning for UE structures in process {}...", pid);
        
        // Ищем GNames
        match scanner.find_gnames(&process) {
            Ok(addr) => {
                println!("[UE] GNames found at: 0x{:x}", addr);
                context.set_variable("gnames_addr", Value::Integer(addr as i64));
            }
            Err(e) => {
                println!("[UE] GNames not found: {}", e);
            }
        }
        
        // Ищем GObjects
        match scanner.find_gobjects(&process) {
            Ok(addr) => {
                println!("[UE] GObjects found at: 0x{:x}", addr);
                context.set_variable("gobjects_addr", Value::Integer(addr as i64));
            }
            Err(e) => {
                println!("[UE] GObjects not found: {}", e);
            }
        }
        
        // Ищем GWorld
        match scanner.find_gworld(&process) {
            Ok(addr) => {
                println!("[UE] GWorld found at: 0x{:x}", addr);
                context.set_variable("gworld_addr", Value::Integer(addr as i64));
            }
            Err(e) => {
                println!("[UE] GWorld not found: {}", e);
            }
        }
        
        Ok(())
    }
    
    fn validate(&self, _step: &StepConfig) -> Result<()> {
        Ok(())
    }
}

/// Шаг для обхода UObject
pub struct UEWalkObjectsStep;

impl Default for UEWalkObjectsStep {
    fn default() -> Self {
        Self
    }
}

impl UEWalkObjectsStep {
    pub fn new() -> Self {
        Self
    }
}

impl StepExecutor for UEWalkObjectsStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        // Интерполируем pid
        let pid = step.parameters.get("pid")
            .and_then(|v| v.as_i64())
            .or_else(|| {
                step.parameters.get("pid")
                    .and_then(|v| v.as_str())
                    .map(|s| context.interpolate_string(s))
                    .and_then(|s| s.parse::<i64>().ok())
            })
            .or_else(|| {
                context.get_variable("last_pid")
                    .ok()
                    .and_then(|v| v.as_i64())
            })
            .or_else(|| {
                context.get_variable("target_pid")
                    .ok()
                    .and_then(|v| v.as_i64())
            })
            .ok_or_else(|| FrameworkError::StepError(
                "UEWalkObjects step requires 'pid' or 'last_pid'".to_string()
            ))?;
        
        let gobjects_addr = context.get_variable("gobjects_addr")?
            .as_i64()
            .ok_or_else(|| FrameworkError::StepError(
                "GObjects address not found. Run UEScan first.".to_string()
            ))?;
        
        let process = ProcessMemory::new(pid as u32)?;
        let mut walker = UObjectWalker::new();
        
        let count = step.parameters.get("max_objects")
            .and_then(|v| v.as_i64())
            .unwrap_or(1000) as usize;
        
        walker.walk(&process, gobjects_addr as u64, count)?;
        
        println!("[UE] Walked {} UObjects", walker.count());
        
        context.set_variable("uobject_count", Value::Integer(walker.count() as i64));
        
        if let Some(store_as) = &step.store_as {
            context.set_variable(store_as, Value::Integer(walker.count() as i64));
        }
        
        Ok(())
    }
    
    fn validate(&self, _step: &StepConfig) -> Result<()> {
        Ok(())
    }
}
