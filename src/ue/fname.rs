// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::error::{FrameworkError, Result};
use crate::memory::MemoryAccessor;
use std::collections::HashMap;

/// FName структура UE4/5
#[derive(Debug, Clone)]
pub struct FName {
    pub index: u32,
    pub number: u32,
    pub name: String,
}

/// FName пул (GNames)
#[derive(Debug)]
pub struct FNamePool {
    pub base_addr: u64,
    pub names: HashMap<u32, String>,
}

impl FNamePool {
    pub fn new(base_addr: u64) -> Self {
        Self {
            base_addr,
            names: HashMap::new(),
        }
    }

    /// Получение имени по индексу
    pub fn get_name(&self, index: u32) -> Option<&str> {
        self.names.get(&index).map(|s| s.as_str())
    }

    /// Добавление имени
    pub fn add_name(&mut self, index: u32, name: String) {
        self.names.insert(index, name);
    }
}

/// Парсер FName
pub struct FNameParser;

impl FNameParser {
    pub fn new() -> Self {
        Self
    }

    /// Парсинг FName из памяти
    pub fn parse_fname(&self, memory: &dyn MemoryAccessor, name_addr: u64) -> Result<FName> {
        // FName в UE4:
        // struct FName {
        //     int32_t Index;      // индекс в GNames
        //     int32_t Number;     // номер (для уникальности)
        // };

        let data = memory.read_bytes(name_addr, 8)?;

        if data.len() < 8 {
            return Err(FrameworkError::MemoryError(format!(
                "Failed to read FName at 0x{:x}",
                name_addr
            )));
        }

        let index = u32::from_le_bytes(data[0..4].try_into().unwrap_or([0; 4]));
        let number = u32::from_le_bytes(data[4..8].try_into().unwrap_or([0; 4]));

        Ok(FName {
            index,
            number,
            name: format!("Name_{}_{}", index, number),
        })
    }

    /// Парсинг строки FName из памяти
    pub fn parse_fname_string(
        &self,
        memory: &dyn MemoryAccessor,
        name_addr: u64,
    ) -> Result<String> {
        let data = memory.read_bytes(name_addr, 32)?;

        if data.is_empty() {
            return Ok(format!("Unknown_{:X}", name_addr));
        }

        // Пробуем прочитать как ASCII строку
        let mut str_end = 0;
        let mut found = false;

        for (i, &byte) in data.iter().enumerate() {
            if byte == 0 {
                str_end = i;
                found = true;
                break;
            }
            // Если байт не ASCII - это не строка
            if !(32..=126).contains(&byte) {
                break;
            }
        }

        if found && str_end > 0 {
            Ok(String::from_utf8_lossy(&data[..str_end]).to_string())
        } else {
            Ok(format!("Name_{:X}", name_addr))
        }
    }
}

impl Default for FNameParser {
    fn default() -> Self {
        Self::new()
    }
}
