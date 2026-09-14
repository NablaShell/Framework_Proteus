// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use crate::error::{FrameworkError, Result};
use crate::memory::MemoryAccessor;
use crate::memory::scanner::AobScanner;
use crate::ue::{GNAMES_PATTERN, GOBJECTS_PATTERN, GWORLD_PATTERN};

/// Сканер UE структур
pub struct UEScanner;

impl UEScanner {
    pub fn new() -> Self {
        Self
    }
    
    /// Поиск GNames
    pub fn find_gnames(&self, memory: &dyn MemoryAccessor) -> Result<u64> {
        let scanner = AobScanner::new(GNAMES_PATTERN)?;
        let results = scanner.scan(memory)?;
        
        if let Some(result) = results.first() {
            // Вычисляем адрес GNames из RIP-relative адресации
            let rip = result.address + 3; // Размер инструкции до displacement
            let disp = i32::from_le_bytes(
                result.pattern[3..7].try_into().unwrap_or([0; 4])
            ) as i64;
            let addr = (rip as i64 + disp) as u64;
            Ok(addr)
        } else {
            Err(FrameworkError::ScanError("GNames not found".to_string()))
        }
    }
    
    /// Поиск GObjects
    pub fn find_gobjects(&self, memory: &dyn MemoryAccessor) -> Result<u64> {
        let scanner = AobScanner::new(GOBJECTS_PATTERN)?;
        let results = scanner.scan(memory)?;
        
        if let Some(result) = results.first() {
            let rip = result.address + 3;
            let disp = i32::from_le_bytes(
                result.pattern[3..7].try_into().unwrap_or([0; 4])
            ) as i64;
            let addr = (rip as i64 + disp) as u64;
            Ok(addr)
        } else {
            Err(FrameworkError::ScanError("GObjects not found".to_string()))
        }
    }
    
    /// Поиск GWorld
    pub fn find_gworld(&self, memory: &dyn MemoryAccessor) -> Result<u64> {
        let scanner = AobScanner::new(GWORLD_PATTERN)?;
        let results = scanner.scan(memory)?;
        
        if let Some(result) = results.first() {
            let rip = result.address + 3;
            let disp = i32::from_le_bytes(
                result.pattern[3..7].try_into().unwrap_or([0; 4])
            ) as i64;
            let addr = (rip as i64 + disp) as u64;
            Ok(addr)
        } else {
            Err(FrameworkError::ScanError("GWorld not found".to_string()))
        }
    }
}

impl Default for UEScanner {
    fn default() -> Self {
        Self::new()
    }
}
