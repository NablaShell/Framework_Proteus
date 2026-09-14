// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use framework_proteus::config::Value;
use framework_proteus::{Config, PipelineExecutor};
use std::process::exit;

fn main() {
    // Инициализация логирования
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <config.yaml>", args[0]);
        exit(1);
    }

    let config_path = &args[1];

    println!("Loading configuration from: {}", config_path);

    let config = match Config::from_file(config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            exit(1);
        }
    };

    let executor = PipelineExecutor::new();

    match executor.execute(&config) {
        Ok(context) => {
            println!("\nFinal variables:");
            for (key, value) in context.get_all_variables() {
                match value {
                    Value::Array(arr) if arr.len() > 10 => {
                        println!("  {} = [{} items]", key, arr.len());
                    }
                    _ => {
                        println!("  {} = {}", key, value.to_string_value());
                    }
                }
            }
            exit(0);
        }
        Err(e) => {
            eprintln!("\nPipeline execution failed: {}", e);
            exit(1);
        }
    }
}
