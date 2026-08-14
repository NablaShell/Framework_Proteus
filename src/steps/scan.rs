// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::config::{StepConfig, Value};
use crate::context::ExecutionContext;
use crate::error::{FrameworkError, Result};
use crate::memory::process::ProcessMemory;
use crate::memory::scanner::AobScanner;
use crate::memory::MemoryAccessor;
use crate::steps::StepExecutor;

pub struct ScanStep;

impl Default for ScanStep {
    fn default() -> Self {
        Self
    }
}

impl ScanStep {
    pub fn new() -> Self {
        Self
    }
}

impl StepExecutor for ScanStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let pattern = step
            .parameters
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(|s| context.interpolate_string(s))
            .ok_or_else(|| {
                FrameworkError::StepError("Scan step requires 'pattern' parameter".to_string())
            })?;

        let pid = step
            .parameters
            .get("pid")
            .and_then(|v| v.as_i64())
            .or_else(|| {
                context
                    .get_variable("last_pid")
                    .ok()
                    .and_then(|v| v.as_i64())
            })
            .ok_or_else(|| {
                FrameworkError::StepError(
                    "Scan step requires 'pid' parameter or 'last_pid' variable".to_string(),
                )
            })?;

        println!("[SCAN] Scanning process {} for pattern '{}'", pid, pattern);

        let mut process = ProcessMemory::new(pid as u32)?;
        process.open()?;

        let scanner = AobScanner::new(&pattern)?;
        let results = scanner.scan(&process)?;

        let addresses: Vec<Value> = results
            .iter()
            .map(|r| Value::Integer(r.address as i64))
            .collect();

        println!(
            "[SCAN] Pattern '{}' found {} matches",
            pattern,
            addresses.len()
        );

        let max_display = step
            .parameters
            .get("max_display")
            .and_then(|v| v.as_i64())
            .unwrap_or(10) as usize;

        for (i, result) in results.iter().take(max_display).enumerate() {
            println!("[SCAN]   Match {}: 0x{:x}", i + 1, result.address);
        }

        if results.len() > max_display {
            println!("[SCAN]   ... and {} more", results.len() - max_display);
        }

        let result_value = Value::Array(addresses);

        if let Some(store_as) = &step.store_as {
            context.set_variable(store_as, result_value.clone());
        }

        context.set_variable(
            &format!("{}_count", step.id),
            Value::Integer(results.len() as i64),
        );

        Ok(())
    }

    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("pattern") {
            return Err(FrameworkError::StepError(
                "Scan step requires 'pattern' parameter".to_string(),
            ));
        }
        Ok(())
    }
}

/// Шаг для сканирования с XOR
pub struct ScanXorStep;

impl Default for ScanXorStep {
    fn default() -> Self {
        Self
    }
}

impl ScanXorStep {
    pub fn new() -> Self {
        Self
    }
}

impl StepExecutor for ScanXorStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let target_value = step
            .parameters
            .get("target_value")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| {
                FrameworkError::StepError(
                    "ScanXor step requires 'target_value' parameter".to_string(),
                )
            })?;

        let pid = step
            .parameters
            .get("pid")
            .and_then(|v| v.as_i64())
            .or_else(|| {
                context
                    .get_variable("last_pid")
                    .ok()
                    .and_then(|v| v.as_i64())
            })
            .ok_or_else(|| {
                FrameworkError::StepError(
                    "ScanXor step requires 'pid' parameter or 'last_pid' variable".to_string(),
                )
            })?;

        println!(
            "[SCAN_XOR] Scanning process {} for XOR pairs (target={})",
            pid, target_value
        );

        let process = ProcessMemory::new(pid as u32)?;

        // Получаем heap регион
        let regions = process.get_memory_regions()?;
        let heap_regions: Vec<_> = regions
            .iter()
            .filter(|r| r.name.as_deref().is_some_and(|n| n.contains("[heap]")))
            .collect();

        let mut found = Vec::new();

        for region in heap_regions {
            println!(
                "[SCAN_XOR] Scanning heap: 0x{:x}-0x{:x}",
                region.start, region.end
            );

            let mut offset = region.start;
            let chunk_size = 4096;

            while offset < region.end {
                let size = std::cmp::min(chunk_size, (region.end - offset) as usize);

                if let Ok(data) = process.read_bytes(offset, size) {
                    // Ищем пары (key, raw_health) где raw XOR key == target
                    for i in 0..data.len().saturating_sub(8) {
                        // Проверяем Python int объекты
                        // Для маленьких int: value = number * 2^3
                        let key_val =
                            u64::from_le_bytes(data[i..i + 8].try_into().unwrap_or([0; 8]));
                        let key = (key_val >> 3) as u8;

                        if key >= 1 {
                            // Ищем рядом raw_health
                            for j in i.saturating_sub(64)
                                ..std::cmp::min(data.len().saturating_sub(8), i + 64)
                            {
                                let raw_val =
                                    u64::from_le_bytes(data[j..j + 8].try_into().unwrap_or([0; 8]));
                                let raw = (raw_val >> 3) as u8;

                                if (raw ^ key) as i64 == target_value {
                                    found.push((offset + i as u64, key, raw));
                                    if found.len() >= 10 {
                                        break;
                                    }
                                }
                            }
                        }

                        if found.len() >= 10 {
                            break;
                        }
                    }
                }

                offset += chunk_size as u64;

                if found.len() >= 10 {
                    break;
                }
            }

            if found.len() >= 10 {
                break;
            }
        }

        println!("[SCAN_XOR] Found {} candidates:", found.len());
        for (addr, key, raw) in &found {
            println!(
                "[SCAN_XOR]   0x{:x}: key={}, raw_health={}, health={}",
                addr,
                key,
                raw,
                raw ^ key
            );
        }

        // Сохраняем результаты
        let addresses: Vec<Value> = found
            .iter()
            .map(|(addr, _, _)| Value::Integer(*addr as i64))
            .collect();

        if let Some(store_as) = &step.store_as {
            context.set_variable(store_as, Value::Array(addresses));
        }

        context.set_variable(
            &format!("{}_count", step.id),
            Value::Integer(found.len() as i64),
        );

        Ok(())
    }

    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("target_value") {
            return Err(FrameworkError::StepError(
                "ScanXor step requires 'target_value' parameter".to_string(),
            ));
        }
        Ok(())
    }
}
