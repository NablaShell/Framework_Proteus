// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::error::{FrameworkError, Result};
use crate::memory::{MemoryAccessor, MemoryRegion, Permissions};
use memmap2::Mmap;
use std::fs::File;

/// Реализация доступа к дампу памяти (файл)
pub struct DumpMemory {
    name: String,
    mmap: Mmap,
    base_address: u64,
    regions: Vec<MemoryRegion>,
}

impl DumpMemory {
    pub fn new(path: &str, base_address: u64) -> Result<Self> {
        let file = File::open(path)
            .map_err(|e| FrameworkError::MemoryError(
                format!("Failed to open dump file {}: {}", path, e)
            ))?;
        
        let mmap = unsafe {
            Mmap::map(&file)
                .map_err(|e| FrameworkError::MemoryError(
                    format!("Failed to mmap dump file: {}", e)
                ))?
        };
        
        let name = format!("dump:{}", path);
        
        // Создаем один регион для всего дампа
        let region = MemoryRegion {
            start: base_address,
            end: base_address + mmap.len() as u64,
            size: mmap.len(),
            permissions: Permissions::new(true, true, false),
            name: Some(path.to_string()),
        };
        
        Ok(Self {
            name,
            mmap,
            base_address,
            regions: vec![region],
        })
    }
    
    pub fn get_base_address(&self) -> u64 {
        self.base_address
    }
    
    pub fn get_size(&self) -> usize {
        self.mmap.len()
    }
}

impl MemoryAccessor for DumpMemory {
    fn read_bytes(&self, address: u64, size: usize) -> Result<Vec<u8>> {
        if address < self.base_address {
            return Err(FrameworkError::MemoryError(
                format!("Address 0x{:x} is below base address 0x{:x}", 
                        address, self.base_address)
            ));
        }
        
        let offset = (address - self.base_address) as usize;
        
        if offset + size > self.mmap.len() {
            return Err(FrameworkError::MemoryError(
                format!("Read at 0x{:x} exceeds dump size", address)
            ));
        }
        
        Ok(self.mmap[offset..offset + size].to_vec())
    }
    
    fn write_bytes(&self, _address: u64, _data: &[u8]) -> Result<()> {
        Err(FrameworkError::MemoryError(
            "Writing to dump files is not supported".to_string()
        ))
    }
    
    fn get_memory_regions(&self) -> Result<Vec<MemoryRegion>> {
        Ok(self.regions.clone())
    }
    
    fn is_readable(&self, address: u64) -> bool {
        address >= self.base_address && 
        address < self.base_address + self.mmap.len() as u64
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
}
