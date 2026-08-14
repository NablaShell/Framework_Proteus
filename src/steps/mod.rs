// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

pub mod print;
pub mod system;
pub mod variables;
pub mod control;
pub mod condition;
pub mod script;
pub mod scan;
pub mod memory;
pub mod process;  
pub mod ue_steps;  

use crate::config::StepConfig;
use crate::context::ExecutionContext;
use crate::error::Result;

pub trait StepExecutor: Send + Sync {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()>;
    fn validate(&self, step: &StepConfig) -> Result<()>;
}

pub struct StepRegistry {
    executors: std::collections::HashMap<String, Box<dyn StepExecutor>>,
}

impl Default for StepRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StepRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            executors: std::collections::HashMap::new(),
        };
        
        registry.register("print", Box::new(print::PrintStep));
        registry.register("system", Box::new(system::SystemStep));
        registry.register("set_variable", Box::new(variables::SetVariableStep));
        registry.register("get_variable", Box::new(variables::GetVariableStep));
        registry.register("sleep", Box::new(control::SleepStep));
        registry.register("log", Box::new(control::LogStep));
        registry.register("if", Box::new(condition::IfStep));
        registry.register("script", Box::new(script::ScriptStep));
        registry.register("lua", Box::new(script::ScriptStep));
        registry.register("spawn_process", Box::new(process::SpawnProcessStep));
        registry.register("run_python", Box::new(process::RunPythonStep));
        registry.register("kill_process", Box::new(process::KillProcessStep));
        
        // Memory steps (требуют настройки)
		// Убираем with_memory, теперь шаги сами создают ProcessMemory
		registry.register("scan", Box::new(scan::ScanStep::new()));
		registry.register("read_memory", Box::new(memory::ReadMemoryStep::new()));
		registry.register("write_memory", Box::new(memory::WriteMemoryStep::new()));
		registry.register("memory_info", Box::new(memory::MemoryInfoStep::new())); 
		registry.register("scan_xor", Box::new(scan::ScanXorStep::new()));

		// EU steps
		registry.register("ue_scan", Box::new(ue_steps::UEScanStep::new()));
		registry.register("ue_walk_objects", Box::new(ue_steps::UEWalkObjectsStep::new()));
		       
        registry
    }
    
    pub fn register(&mut self, name: &str, executor: Box<dyn StepExecutor>) {
        self.executors.insert(name.to_string(), executor);
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn StepExecutor> {
        self.executors.get(name).map(|e| e.as_ref())
    }
}
