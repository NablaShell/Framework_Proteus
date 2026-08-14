// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::error::{FrameworkError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub target: Option<TargetConfig>,
    pub pipeline: Vec<StepConfig>,
    pub variables: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    pub name: String,
    pub mode: TargetMode,
    #[serde(default)]
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetMode {
    Live,
    Dump,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    pub id: String,
    pub name: String,
    pub action: ActionType,
    #[serde(default)]
    pub parameters: HashMap<String, Value>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub store_as: Option<String>,
    #[serde(default)]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Print,
    System,
    SetVariable,
    GetVariable,
    Sleep,
    Log,
    If,
    Script,
    Lua,
    Scan,
    ReadMemory,
    WriteMemory,
    MemoryInfo,
    SpawnProcess,
    RunPython,
    KillProcess,
    ScanXor,
    #[serde(rename = "ue_scan")]
    UEScan,
    #[serde(rename = "ue_walk_objects")]
    UEWalkObjects,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let action_str = match self {
            ActionType::Print => "print",
            ActionType::System => "system",
            ActionType::SetVariable => "set_variable",
            ActionType::GetVariable => "get_variable",
            ActionType::Sleep => "sleep",
            ActionType::Log => "log",
            ActionType::If => "if",
            ActionType::Script => "script",
            ActionType::Lua => "lua",
            ActionType::Scan => "scan",
            ActionType::ReadMemory => "read_memory",
            ActionType::WriteMemory => "write_memory",
            ActionType::MemoryInfo => "memory_info",
            ActionType::SpawnProcess => "spawn_process",
            ActionType::RunPython => "run_python",
            ActionType::KillProcess => "kill_process",
            ActionType::ScanXor => "scan_xor",
            ActionType::UEScan => "ue_scan",
            ActionType::UEWalkObjects => "ue_walk_objects",
        };
        write!(f, "{}", action_str)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn to_string_value(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(arr) => format!("{:?}", arr),
            Value::Object(obj) => format!("{:?}", obj),
            Value::Null => "null".to_string(),
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self> {
        let config: Config = serde_yaml::from_str(content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != "1.0" {
            return Err(FrameworkError::ConfigError(format!(
                "Unsupported version: {}",
                self.version
            )));
        }

        // Проверка на дублирующиеся ID
        let mut ids = std::collections::HashSet::new();
        for step in &self.pipeline {
            if !ids.insert(&step.id) {
                return Err(FrameworkError::ConfigError(format!(
                    "Duplicate step ID: {}",
                    step.id
                )));
            }
        }

        // Проверка зависимостей
        let all_ids: Vec<&str> = self.pipeline.iter().map(|s| s.id.as_str()).collect();
        for step in &self.pipeline {
            for dep in &step.depends_on {
                if !all_ids.contains(&dep.as_str()) {
                    return Err(FrameworkError::ConfigError(format!(
                        "Step '{}' depends on non-existent step '{}'",
                        step.id, dep
                    )));
                }
            }
        }

        // Проверка на циклы
        self.check_cycles()?;

        Ok(())
    }

    fn check_cycles(&self) -> Result<()> {
        // Простая проверка циклов через DFS
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();

        // Строим граф зависимостей
        for step in &self.pipeline {
            graph.entry(&step.id).or_default();
            for dep in &step.depends_on {
                graph.entry(dep.as_str()).or_default().push(&step.id);
            }
        }

        // DFS для поиска циклов
        fn has_cycle<'a>(
            node: &'a str,
            graph: &HashMap<&'a str, Vec<&'a str>>,
            visited: &mut HashMap<&'a str, bool>,
            rec_stack: &mut HashMap<&'a str, bool>,
        ) -> bool {
            visited.insert(node, true);
            rec_stack.insert(node, true);

            if let Some(neighbors) = graph.get(node) {
                for &neighbor in neighbors {
                    if !*visited.get(neighbor).unwrap_or(&false) {
                        if has_cycle(neighbor, graph, visited, rec_stack) {
                            return true;
                        }
                    } else if *rec_stack.get(neighbor).unwrap_or(&false) {
                        return true;
                    }
                }
            }

            rec_stack.insert(node, false);
            false
        }

        let mut visited = HashMap::new();
        let mut rec_stack = HashMap::new();

        for step in &self.pipeline {
            if !*visited.get(step.id.as_str()).unwrap_or(&false)
                && has_cycle(&step.id, &graph, &mut visited, &mut rec_stack)
            {
                return Err(FrameworkError::ConfigError(format!(
                    "Cyclic dependency detected involving step '{}'",
                    step.id
                )));
            }
        }

        Ok(())
    }
}

// Реализуем FromStr вместо from_str
impl FromStr for Config {
    type Err = FrameworkError;

    fn from_str(content: &str) -> Result<Self> {
        Self::parse(content)
    }
}
