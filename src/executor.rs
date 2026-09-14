// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use std::time::Instant;
use crate::config::Config;
use crate::context::{ExecutionContext, StepResult};
use crate::dag::builder::DagBuilder;
use crate::dag::scheduler::DagScheduler;
use crate::dag::validator::DagValidator;
use crate::error::{FrameworkError, Result};
use crate::steps::StepRegistry;

pub struct PipelineExecutor {
    registry: StepRegistry,
}

impl Default for PipelineExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineExecutor {
    pub fn new() -> Self {
        Self {
            registry: StepRegistry::new(),
        }
    }
    
    pub fn execute(&self, config: &Config) -> Result<ExecutionContext> {
        // Валидация DAG
        DagValidator::validate(config)?;
        
        // Построение DAG
        let mut dag = DagBuilder::new();
        dag.build(config)?;
        
        // Выполнение через планировщик
        let scheduler = DagScheduler::new(self.registry.clone());
        scheduler.execute(config, &dag)
    }
    
    pub fn execute_sequential(&self, config: &Config) -> Result<ExecutionContext> {
        // Для обратной совместимости - последовательное выполнение
        let mut context = ExecutionContext::new();
        
        if let Some(variables) = &config.variables {
            for (key, value) in variables {
                context.set_variable(key, value.clone());
            }
        }
        
        println!("=== Starting Sequential Pipeline Execution ===");
        
        for (index, step) in config.pipeline.iter().enumerate() {
            println!("[{}/{}] Executing: {} ({})", index + 1, config.pipeline.len(), step.name, step.id);
            
            let executor = self.registry.get(&step.action.to_string())
                .ok_or_else(|| FrameworkError::StepError(
                    format!("No executor found for action: {:?}", step.action)
                ))?;
            
            executor.validate(step)?;
            
            let start_time = Instant::now();
            let result = executor.execute(step, &mut context);
            let duration = start_time.elapsed();
            
            match result {
                Ok(()) => {
                    println!("[SUCCESS] Step '{}' completed in {:?}", step.name, duration);
                    context.set_step_result(&step.id, StepResult {
                        success: true,
                        output: None,
                        duration,
                        message: None,
                    });
                }
                Err(e) => {
                    eprintln!("[ERROR] Step '{}' failed: {}", step.name, e);
                    return Err(e);
                }
            }
            println!();
        }
        
        Ok(context)
    }
}

// Добавляем Clone для StepRegistry
impl Clone for StepRegistry {
    fn clone(&self) -> Self {
        // В реальном коде нужно использовать Arc для исполнителей
        // Для MVP создаем новый реестр
        Self::new()
    }
}
