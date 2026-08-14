// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use std::collections::HashMap;
use crate::config::Config;
use crate::error::{FrameworkError, Result};

#[derive(Debug)]
pub struct DagBuilder {
    graph: DiGraph<String, ()>,
    step_map: HashMap<String, NodeIndex>,
    execution_order: Vec<Vec<String>>,
}

impl DagBuilder {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            step_map: HashMap::new(),
            execution_order: Vec::new(),
        }
    }
    
    pub fn build(&mut self, config: &Config) -> Result<()> {
        // Добавляем все шаги как узлы
        for step in &config.pipeline {
            let node_idx = self.graph.add_node(step.id.clone());
            self.step_map.insert(step.id.clone(), node_idx);
        }
        
        // Добавляем ребра для явных зависимостей
        for step in &config.pipeline {
            if let Some(from_idx) = self.step_map.get(&step.id) {
                for dep in &step.depends_on {
                    if let Some(to_idx) = self.step_map.get(dep) {
                        // Ребро от зависимости к зависимому шагу
                        self.graph.add_edge(*to_idx, *from_idx, ());
                    }
                }
            }
        }
        
        // Добавляем ребра для зависимостей по данным (data dependencies)
        self.add_data_dependencies(config);
        
        // Вычисляем порядок выполнения
        self.calculate_execution_order()?;
        
        Ok(())
    }
    
    fn add_data_dependencies(&mut self, config: &Config) {
        // Находим зависимости по переменным
        // Шаг A зависит от шага B, если A использует переменную, которую B создает
        
        let mut variable_producers: HashMap<String, String> = HashMap::new();
        
        for step in &config.pipeline {
            // Проверяем, какие переменные производит шаг
            let produced_vars = self.get_produced_variables(step);
            
            // Проверяем, какие переменные использует шаг
            let used_vars = self.get_used_variables(step);
            
            // Для каждой используемой переменной ищем производителя
            for var in used_vars {
                if let Some(producer_id) = variable_producers.get(&var) {
                    // Добавляем зависимость, если её еще нет
                    if producer_id != &step.id {
                        if let (Some(from_idx), Some(to_idx)) = (
                            self.step_map.get(producer_id),
                            self.step_map.get(&step.id)
                        ) {
                            // Проверяем, что ребра еще нет
                            if !self.graph.contains_edge(*from_idx, *to_idx) {
                                self.graph.add_edge(*from_idx, *to_idx, ());
                            }
                        }
                    }
                }
            }
            
            // Регистрируем производимые переменные
            for var in produced_vars {
                variable_producers.insert(var, step.id.clone());
            }
        }
    }
    
    fn get_produced_variables(&self, step: &crate::config::StepConfig) -> Vec<String> {
        let mut vars = Vec::new();
        
        // Шаг с store_as производит переменную
        if let Some(store_as) = &step.store_as {
            vars.push(store_as.clone());
        }
        
        // set_variable производит переменную из параметров
        if step.action.to_string() == "set_variable" {
            if let Some(name) = step.parameters.get("name") {
                if let Some(name_str) = name.as_str() {
                    vars.push(name_str.to_string());
                }
            }
        }
        
        vars
    }
    
    fn get_used_variables(&self, step: &crate::config::StepConfig) -> Vec<String> {
        let mut vars = Vec::new();
        
        // Проверяем все строковые параметры на использование переменных
        for value in step.parameters.values() {
            self.extract_variables_from_value(value, &mut vars);
        }
        
        // Проверяем условие
        if let Some(condition) = &step.condition {
            self.extract_variables_from_string(condition, &mut vars);
        }
        
        vars
    }
    
    fn extract_variables_from_value(&self, value: &crate::config::Value, vars: &mut Vec<String>) {
        match value {
            crate::config::Value::String(s) => {
                self.extract_variables_from_string(s, vars);
            }
            crate::config::Value::Array(arr) => {
                for item in arr {
                    self.extract_variables_from_value(item, vars);
                }
            }
            crate::config::Value::Object(obj) => {
                for item in obj.values() {
                    self.extract_variables_from_value(item, vars);
                }
            }
            _ => {}
        }
    }
    
    fn extract_variables_from_string(&self, input: &str, vars: &mut Vec<String>) {
        let mut start_idx = None;
        
        for (idx, ch) in input.char_indices() {
            if ch == '$' && input[idx..].starts_with("${") {
                start_idx = Some(idx);
            } else if ch == '}' {
                if let Some(start) = start_idx {
                    let var_name = &input[start + 2..idx];
                    if !var_name.is_empty() && !vars.contains(&var_name.to_string()) {
                        vars.push(var_name.to_string());
                    }
                    start_idx = None;
                }
            }
        }
    }
    
    fn calculate_execution_order(&mut self) -> Result<()> {
        // Топологическая сортировка
        let topo_order = toposort(&self.graph, None)
            .map_err(|cycle| FrameworkError::ConfigError(
                format!("Cyclic dependency detected: {:?}", cycle)
            ))?;
        
        // Группируем шаги по уровням для параллельного выполнения
        let mut levels: HashMap<NodeIndex, usize> = HashMap::new();
        
        // Вычисляем уровень каждого узла (максимальная длина пути от корня)
        for node_idx in &topo_order {
            let mut max_level = 0;
            
            // Проверяем всех предшественников
            for pred in self.graph.neighbors_directed(*node_idx, petgraph::Direction::Incoming) {
                if let Some(pred_level) = levels.get(&pred) {
                    max_level = max_level.max(*pred_level + 1);
                }
            }
            
            levels.insert(*node_idx, max_level);
        }
        
        // Находим максимальный уровень
        let max_level = levels.values().max().copied().unwrap_or(0);
        
        // Группируем узлы по уровням
        let mut level_groups: Vec<Vec<String>> = vec![Vec::new(); max_level + 1];
        
        for (node_idx, level) in &levels {
            if let Some(step_id) = self.graph.node_weight(*node_idx) {
                level_groups[*level].push(step_id.clone());
            }
        }
        
        // Сохраняем порядок выполнения
        self.execution_order = level_groups;
        
        Ok(())
    }
    
    pub fn get_execution_order(&self) -> &[Vec<String>] {
        &self.execution_order
    }
    
    pub fn get_parallel_groups_count(&self) -> usize {
        self.execution_order.len()
    }
    
    pub fn get_max_parallelism(&self) -> usize {
        self.execution_order.iter().map(|group| group.len()).max().unwrap_or(0)
    }
    
    pub fn validate_acyclic(&self) -> bool {
        toposort(&self.graph, None).is_ok()
    }
}

impl Default for DagBuilder {
    fn default() -> Self {
        Self::new()
    }
}
