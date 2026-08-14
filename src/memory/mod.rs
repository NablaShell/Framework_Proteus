// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

pub mod process;
pub mod dump;
pub mod scanner;

use crate::error::Result;
use std::ops::Range;

/// Основной трейт для доступа к памяти
pub trait MemoryAccessor: Send + Sync {
    /// Читает байты из памяти по указанному адресу
    fn read_bytes(&self, address: u64, size: usize) -> Result<Vec<u8>>;
    
    /// Записывает байты в память по указанному адресу
    fn write_bytes(&self, address: u64, data: &[u8]) -> Result<()>;
    
    /// Возвращает диапазоны доступной памяти
    fn get_memory_regions(&self) -> Result<Vec<MemoryRegion>>;
    
    /// Проверяет, доступен ли адрес для чтения
    fn is_readable(&self, address: u64) -> bool;
    
    /// Возвращает имя процесса или файла
    fn get_name(&self) -> &str;
}

/// Информация о регионе памяти
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
    pub size: usize,
    pub permissions: Permissions,
    pub name: Option<String>,
}

impl MemoryRegion {
    pub fn range(&self) -> Range<u64> {
        self.start..self.end
    }
    
    pub fn contains(&self, address: u64) -> bool {
        address >= self.start && address < self.end
    }
}

/// Права доступа к региону памяти
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Permissions {
    pub fn new(read: bool, write: bool, execute: bool) -> Self {
        Self { read, write, execute }
    }
    
    pub fn is_readable(&self) -> bool {
        self.read
    }
    
    pub fn is_writable(&self) -> bool {
        self.write
    }
    
    pub fn is_executable(&self) -> bool {
        self.execute
    }
}

/// Результат поиска паттерна
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub address: u64,
    pub pattern: Vec<u8>,
    pub region: Option<MemoryRegion>,
}
