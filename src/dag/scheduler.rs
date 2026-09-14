// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use rayon::prelude::*;
use crate::config::Config;
use crate::context::ExecutionContext;
use crate::dag::builder::DagBuilder;
use crate::error::{FrameworkError, Result};
use crate::steps::StepRegistry;

pub struct DagScheduler {
    registry: Arc<StepRegistry>,
}

impl DagScheduler {
    pub fn new(registry: StepRegistry) -> Self {
        Self {
            registry: Arc::new(registry),
        }
    }
    
    pub fn execute(&self, config: &Config, dag: &DagBuilder) -> Result<ExecutionContext> {
        let context = Arc::new(RwLock::new(ExecutionContext::new()));
        
        // Устанавливаем глобальные переменные
        if let Some(variables) = &config.variables {
            let mut ctx = context.write();
            for (key, value) in variables {
                ctx.set_variable(key, value.clone());
            }
        }
        
        println!("=== Starting DAG Pipeline Execution ===");
        println!("Execution order: {} parallel groups", dag.get_parallel_groups_count());
        println!("Max parallelism: {} steps", dag.get_max_parallelism());
        println!();
        
        let step_map: HashMap<&str, &crate::config::StepConfig> = config.pipeline
            .iter()
            .map(|step| (step.id.as_str(), step))
            .collect();
        
        let mut completed_steps = 0;
        let total_steps = config.pipeline.len();
        
        // Выполняем шаги по уровням
        for (level_idx, level_group) in dag.get_execution_order().iter().enumerate() {
            println!("[Level {}/{}] Executing {} step(s)", 
                     level_idx + 1, 
                     dag.get_parallel_groups_count(),
                     level_group.len());
            
            // Выполняем шаги на текущем уровне параллельно
            let results: Vec<Result<()>> = level_group
                .par_iter()
                .map(|step_id| {
                    if let Some(step) = step_map.get(step_id.as_str()) {
                        self.execute_step(step, &context)
                    } else {
                        Err(FrameworkError::StepError(
                            format!("Step '{}' not found in configuration", step_id)
                        ))
                    }
                })
                .collect();
            
            // Проверяем результаты
            let mut has_errors = false;
            for (step_id, result) in level_group.iter().zip(results) {
                match result {
                    Ok(()) => {
                        completed_steps += 1;
                        println!("[SUCCESS] Step '{}' completed", step_id);
                    }
                    Err(e) => {
                        has_errors = true;
                        eprintln!("[ERROR] Step '{}' failed: {}", step_id, e);
                        
                        // Логируем ошибку в контекст
                        let mut ctx = context.write();
                        ctx.set_step_result(step_id, crate::context::StepResult {
                            success: false,
                            output: None,
                            duration: std::time::Duration::from_secs(0),
                            message: Some(e.to_string()),
                        });
                    }
                }
            }
            
            if has_errors {
                return Err(FrameworkError::StepError(
                    format!("Pipeline failed at level {}", level_idx + 1)
                ));
            }
            
            println!("[Level {}/{}] Completed", level_idx + 1, dag.get_parallel_groups_count());
            println!("Progress: {}/{} steps", completed_steps, total_steps);
            println!();
        }
        
        println!("=== DAG Pipeline Execution Completed ===");
        
        // Возвращаем контекст
        let context = Arc::try_unwrap(context)
            .map_err(|_| FrameworkError::ContextError("Failed to unwrap context".to_string()))?
            .into_inner();
        
        Ok(context)
    }
    
    fn execute_step(
        &self,
        step: &crate::config::StepConfig,
        context: &Arc<RwLock<ExecutionContext>>,
    ) -> Result<()> {
        // Проверяем условие выполнения
        if let Some(condition) = &step.condition {
            let ctx = context.read();
            let condition_result = self.evaluate_condition(condition, &ctx)?;
            drop(ctx); // Освобождаем блокировку
            
            if !condition_result {
                println!("[SKIP] Step '{}' skipped (condition not met)", step.name);
                return Ok(());
            }
        }
        
        // Получаем исполнителя для действия
        let executor = self.registry.get(&step.action.to_string())
            .ok_or_else(|| FrameworkError::StepError(
                format!("No executor found for action: {:?}", step.action)
            ))?;
        
        // Валидация шага
        executor.validate(step)?;
        
        // Выполнение с таймаутом
        let start = std::time::Instant::now();
        
        // Блокируем контекст на время выполнения
        let mut ctx = context.write();
        let result = executor.execute(step, &mut ctx);
        let duration = start.elapsed();
        
        // Проверяем таймаут
        if let Some(timeout_ms) = step.timeout {
            if duration.as_millis() > timeout_ms as u128 {
                return Err(FrameworkError::StepError(
                    format!("Step '{}' timed out after {}ms (limit: {}ms)", 
                            step.name, duration.as_millis(), timeout_ms)
                ));
            }
        }
        
        // Сохраняем результат
        let step_result = match &result {
            Ok(()) => crate::context::StepResult {
                success: true,
                output: None,
                duration,
                message: None,
            },
            Err(e) => crate::context::StepResult {
                success: false,
                output: None,
                duration,
                message: Some(e.to_string()),
            },
        };
        
        ctx.set_step_result(&step.id, step_result);
        
        result
    }
    
    fn evaluate_condition(&self, condition: &str, context: &ExecutionContext) -> Result<bool> {
        // Простая оценка условий вида "${variable} == value"
        let interpolated = context.interpolate_string(condition);
        
        // Поддерживаемые операторы
        let operators = ["==", "!=", ">=", "<=", ">", "<"];
        
        for op in operators {
            if let Some(pos) = interpolated.find(op) {
                let left = interpolated[..pos].trim();
                let right = interpolated[pos + op.len()..].trim();
                
                return match op {
                    "==" => Ok(left == right),
                    "!=" => Ok(left != right),
                    ">=" => Ok(left >= right),
                    "<=" => Ok(left <= right),
                    ">" => Ok(left > right),
                    "<" => Ok(left < right),
                    _ => Ok(false),
                };
            }
        }
        
        // Если нет оператора, проверяем на true/false
        Ok(matches!(interpolated.to_lowercase().as_str(), "true" | "yes" | "1"))
    }
}
