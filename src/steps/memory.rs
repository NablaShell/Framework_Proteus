// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::config::{StepConfig, Value};
use crate::context::ExecutionContext;
use crate::error::{FrameworkError, Result};
use crate::memory::process::ProcessMemory;
use crate::memory::MemoryAccessor;
use crate::steps::StepExecutor;

/// Шаг для чтения памяти
pub struct ReadMemoryStep;

impl Default for ReadMemoryStep {
    fn default() -> Self {
        Self
    }
}

impl ReadMemoryStep {
    pub fn new() -> Self {
        Self
    }

    fn resolve_address(step: &StepConfig, context: &ExecutionContext) -> Result<u64> {
        if let Some(addr_value) = step.parameters.get("address") {
            match addr_value {
                Value::Integer(i) => Ok(*i as u64),
                Value::Array(arr) => {
                    // Берем первый элемент массива
                    arr.first()
                        .and_then(|v| v.as_i64())
                        .map(|v| v as u64)
                        .ok_or_else(|| {
                            FrameworkError::StepError("Address array is empty".to_string())
                        })
                }
                Value::String(s) => {
                    // Проверяем, есть ли прямая индексация
                    if s.contains("${") {
                        // Извлекаем имя переменной и индекс
                        // Например: ${flag_addresses[0]}
                        let content = s.trim_start_matches("${").trim_end_matches("}");

                        if let Some(bracket_pos) = content.find('[') {
                            let var_name = &content[..bracket_pos];
                            let index_str = &content[bracket_pos + 1..content.len() - 1];

                            if let Ok(index) = index_str.parse::<usize>() {
                                // Получаем массив из контекста
                                if let Ok(value) = context.get_variable(var_name) {
                                    if let Value::Array(arr) = value {
                                        if let Some(item) = arr.get(index) {
                                            return item.as_i64().map(|v| v as u64).ok_or_else(
                                                || {
                                                    FrameworkError::StepError(format!(
                                                        "Array element {} is not an integer",
                                                        index
                                                    ))
                                                },
                                            );
                                        } else {
                                            return Err(FrameworkError::StepError(format!(
                                                "Array index {} out of bounds",
                                                index
                                            )));
                                        }
                                    } else {
                                        return Err(FrameworkError::StepError(format!(
                                            "Variable '{}' is not an array",
                                            var_name
                                        )));
                                    }
                                } else {
                                    return Err(FrameworkError::StepError(format!(
                                        "Variable '{}' not found",
                                        var_name
                                    )));
                                }
                            }
                        }
                    }

                    // Интерполируем обычную строку
                    let interpolated = context.interpolate_string(s);

                    // Пробуем распарсить как число
                    interpolated.parse::<u64>().map_err(|e| {
                        FrameworkError::StepError(format!(
                            "Invalid address '{}': {}",
                            interpolated, e
                        ))
                    })
                }
                _ => Err(FrameworkError::StepError(
                    "Invalid address type".to_string(),
                )),
            }
        } else {
            Err(FrameworkError::StepError(
                "ReadMemory step requires 'address' parameter".to_string(),
            ))
        }
    }
}

impl StepExecutor for ReadMemoryStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let address = Self::resolve_address(step, context)?;

        let size = step
            .parameters
            .get("size")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| {
                FrameworkError::StepError("ReadMemory step requires 'size' parameter".to_string())
            })?;

        // Получаем PID
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
                    "ReadMemory step requires 'pid' parameter or 'last_pid' variable".to_string(),
                )
            })?;

        let process = ProcessMemory::new(pid as u32)?;

        // Читаем память
        let data = process.read_bytes(address, size as usize)?;

        println!("[MEMORY] Read {} bytes from 0x{:x}", data.len(), address);

        // Сохраняем результат
        let hex_data = hex::encode(&data);
        let ascii_data = String::from_utf8_lossy(&data).to_string();

        println!("[MEMORY] ASCII: {}", ascii_data);

        if let Some(store_as) = &step.store_as {
            context.set_variable(store_as, Value::String(hex_data));
        }

        // Также сохраняем ASCII версию
        context.set_variable("last_read_ascii", Value::String(ascii_data));

        Ok(())
    }

    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("address") {
            return Err(FrameworkError::StepError(
                "ReadMemory step requires 'address' parameter".to_string(),
            ));
        }
        if !step.parameters.contains_key("size") {
            return Err(FrameworkError::StepError(
                "ReadMemory step requires 'size' parameter".to_string(),
            ));
        }
        Ok(())
    }
}

/// Шаг для записи в память
pub struct WriteMemoryStep;

impl Default for WriteMemoryStep {
    fn default() -> Self {
        Self
    }
}

impl WriteMemoryStep {
    pub fn new() -> Self {
        Self
    }
}

impl StepExecutor for WriteMemoryStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        let address = step
            .parameters
            .get("address")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| {
                FrameworkError::StepError(
                    "WriteMemory step requires 'address' parameter".to_string(),
                )
            })?;

        let data_hex = step
            .parameters
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FrameworkError::StepError("WriteMemory step requires 'data' parameter".to_string())
            })?;

        // Декодируем hex
        let data = hex::decode(data_hex.trim())
            .map_err(|e| FrameworkError::StepError(format!("Invalid hex data: {}", e)))?;

        // Получаем PID
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
                    "WriteMemory step requires 'pid' parameter or 'last_pid' variable".to_string(),
                )
            })?;

        let process = ProcessMemory::new(pid as u32)?;

        // Записываем в память
        process.write_bytes(address as u64, &data)?;

        println!("[MEMORY] Wrote {} bytes to 0x{:x}", data.len(), address);

        Ok(())
    }

    fn validate(&self, step: &StepConfig) -> Result<()> {
        if !step.parameters.contains_key("address") {
            return Err(FrameworkError::StepError(
                "WriteMemory step requires 'address' parameter".to_string(),
            ));
        }
        if !step.parameters.contains_key("data") {
            return Err(FrameworkError::StepError(
                "WriteMemory step requires 'data' parameter".to_string(),
            ));
        }
        Ok(())
    }
}

/// Шаг для получения информации о регионах памяти
pub struct MemoryInfoStep;

impl Default for MemoryInfoStep {
    fn default() -> Self {
        Self
    }
}

impl MemoryInfoStep {
    pub fn new() -> Self {
        Self
    }
}

impl StepExecutor for MemoryInfoStep {
    fn execute(&self, step: &StepConfig, context: &mut ExecutionContext) -> Result<()> {
        // Получаем PID
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
                    "MemoryInfo step requires 'pid' parameter or 'last_pid' variable".to_string(),
                )
            })?;

        let process = ProcessMemory::new(pid as u32)?;
        let regions = process.get_memory_regions()?;

        println!("[MEMORY] Memory regions for {}:", process.get_name());
        println!("[MEMORY] Total regions: {}", regions.len());

        let total_size: usize = regions.iter().map(|r| r.size).sum();
        println!(
            "[MEMORY] Total size: {:.2} MB",
            total_size as f64 / 1024.0 / 1024.0
        );

        // Выводим первые несколько регионов
        let max_display = step
            .parameters
            .get("max_display")
            .and_then(|v| v.as_i64())
            .unwrap_or(20) as usize;

        for (i, region) in regions.iter().take(max_display).enumerate() {
            println!(
                "[MEMORY]   Region {}: 0x{:x}-0x{:x} ({}) {}{}{} {}",
                i + 1,
                region.start,
                region.end,
                region.size,
                if region.permissions.read { "r" } else { "-" },
                if region.permissions.write { "w" } else { "-" },
                if region.permissions.execute { "x" } else { "-" },
                region.name.as_deref().unwrap_or("")
            );
        }

        if regions.len() > max_display {
            println!("[MEMORY]   ... and {} more", regions.len() - max_display);
        }

        // Сохраняем количество регионов
        context.set_variable(
            &format!("{}_region_count", step.id),
            Value::Integer(regions.len() as i64),
        );

        Ok(())
    }

    fn validate(&self, _step: &StepConfig) -> Result<()> {
        Ok(())
    }
}
