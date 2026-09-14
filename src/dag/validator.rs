// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use std::collections::HashMap;
use crate::config::Config;
use crate::error::{FrameworkError, Result};

pub struct DagValidator;

impl DagValidator {
    pub fn validate(config: &Config) -> Result<()> {
        // Проверка на дублирующиеся ID
        let mut ids = HashMap::new();
        for step in &config.pipeline {
            if ids.insert(&step.id, step).is_some() {
                return Err(FrameworkError::ConfigError(
                    format!("Duplicate step ID: {}", step.id)
                ));
            }
        }
        
        // Проверка зависимостей
        let all_ids: Vec<&str> = config.pipeline.iter().map(|s| s.id.as_str()).collect();
        for step in &config.pipeline {
            for dep in &step.depends_on {
                if !all_ids.contains(&dep.as_str()) {
                    return Err(FrameworkError::ConfigError(
                        format!("Step '{}' depends on non-existent step '{}'", step.id, dep)
                    ));
                }
            }
        }
        
        // Проверка на циклы
        Self::check_cycles(config)
    }
    
    fn check_cycles(config: &Config) -> Result<()> {
        let mut graph = DiGraph::<String, ()>::new();
        let mut node_map = HashMap::new();
        
        // Добавляем узлы
        for step in &config.pipeline {
            let idx = graph.add_node(step.id.clone());
            node_map.insert(step.id.clone(), idx);
        }
        
        // Добавляем ребра (от зависимости к зависимому)
        for step in &config.pipeline {
            if let Some(to_idx) = node_map.get(&step.id) {
                for dep in &step.depends_on {
                    if let Some(from_idx) = node_map.get(dep) {
                        graph.add_edge(*from_idx, *to_idx, ());
                    }
                }
            }
        }
        
        // Пытаемся выполнить топологическую сортировку
        match toposort(&graph, None) {
            Ok(_) => Ok(()),
            Err(_) => {
                // Находим циклические узлы через DFS
                let cyclic_nodes = Self::find_cycle_nodes(&graph);
                
                Err(FrameworkError::ConfigError(
                    format!("Cyclic dependency detected involving steps: {:?}", cyclic_nodes)
                ))
            }
        }
    }
    
    fn find_cycle_nodes(graph: &DiGraph<String, ()>) -> Vec<String> {
        // Простой поиск циклов через DFS
        let mut visited = vec![false; graph.node_count()];
        let mut recursion_stack = vec![false; graph.node_count()];
        let mut cycle_nodes = Vec::new();
        
        for start_idx in graph.node_indices() {
            if !visited[start_idx.index()] {
                Self::dfs_cycle_detection(
                    graph,
                    start_idx,
                    &mut visited,
                    &mut recursion_stack,
                    &mut cycle_nodes,
                );
            }
        }
        
        if cycle_nodes.is_empty() {
            // Если не нашли через DFS, пробуем найти хотя бы один узел
            for idx in graph.node_indices() {
                if let Some(weight) = graph.node_weight(idx) {
                    cycle_nodes.push(weight.clone());
                }
            }
        }
        
        cycle_nodes
    }
    
    fn dfs_cycle_detection(
        graph: &DiGraph<String, ()>,
        node: petgraph::graph::NodeIndex,
        visited: &mut [bool],
        recursion_stack: &mut [bool],
        cycle_nodes: &mut Vec<String>,
    ) {
        let node_idx = node.index();
        visited[node_idx] = true;
        recursion_stack[node_idx] = true;
        
        for neighbor in graph.neighbors_directed(node, petgraph::Direction::Outgoing) {
            let neighbor_idx = neighbor.index();
            
            if !visited[neighbor_idx] {
                Self::dfs_cycle_detection(
                    graph,
                    neighbor,
                    visited,
                    recursion_stack,
                    cycle_nodes,
                );
            } else if recursion_stack[neighbor_idx] {
                // Нашли цикл
                if let Some(weight) = graph.node_weight(neighbor) {
                    if !cycle_nodes.contains(weight) {
                        cycle_nodes.push(weight.clone());
                    }
                }
            }
        }
        
        recursion_stack[node_idx] = false;
    }
}
