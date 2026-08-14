// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use crate::error::{FrameworkError, Result};
use crate::memory::{MemoryAccessor, MemoryRegion, Permissions};
use std::fs::File;

/// Реализация доступа к памяти живого процесса через /proc/pid/mem (Linux)
pub struct ProcessMemory {
    pid: u32,
    name: String,
    mem_file: Option<File>,
    regions: Vec<MemoryRegion>,
}

impl ProcessMemory {
    pub fn new(pid: u32) -> Result<Self> {
        let name = Self::get_process_name(pid)?;
        let regions = Self::parse_maps(pid)?;
        
        Ok(Self {
            pid,
            name,
            mem_file: None,
            regions,
        })
    }
    
    pub fn open(&mut self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let mem_path = format!("/proc/{}/mem", self.pid);
            let file = File::open(&mem_path)
                .map_err(|e| FrameworkError::ProcessError(
                    format!("Failed to open {}: {}", mem_path, e)
                ))?;
            self.mem_file = Some(file);
        }
        
        Ok(())
    }
    
    pub fn close(&mut self) {
        self.mem_file = None;
    }
    
    pub fn get_pid(&self) -> u32 {
        self.pid
    }
    
    fn get_process_name(pid: u32) -> Result<String> {
        #[cfg(target_os = "linux")]
        {
            let comm_path = format!("/proc/{}/comm", pid);
            let name = std::fs::read_to_string(&comm_path)
                .map_err(|e| FrameworkError::ProcessError(
                    format!("Failed to read {}: {}", comm_path, e)
                ))?
                .trim()
                .to_string();
            Ok(name)
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Ok(format!("process_{}", pid))
        }
    }
    
    fn parse_maps(pid: u32) -> Result<Vec<MemoryRegion>> {
        #[cfg(target_os = "linux")]
        {
            let maps_path = format!("/proc/{}/maps", pid);
            let content = std::fs::read_to_string(&maps_path)
                .map_err(|e| FrameworkError::ProcessError(
                    format!("Failed to read {}: {}", maps_path, e)
                ))?;
            
            let mut regions = Vec::new();
            
            for line in content.lines() {
                if let Some(region) = parse_maps_line(line) {
                    regions.push(region);
                }
            }
            
            Ok(regions)
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Ok(Vec::new())
        }
    }
    
    pub fn find_region(&self, address: u64) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.contains(address))
    }
    
    pub fn get_regions_with_permission(&self, perm: &Permissions) -> Vec<&MemoryRegion> {
        self.regions.iter()
            .filter(|r| {
                (perm.read && r.permissions.read) ||
                (perm.write && r.permissions.write) ||
                (perm.execute && r.permissions.execute)
            })
            .collect()
    }
}

impl MemoryAccessor for ProcessMemory {
    fn read_bytes(&self, address: u64, size: usize) -> Result<Vec<u8>> {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::FileExt;
            
            let mem_path = format!("/proc/{}/mem", self.pid);
            let file = File::open(&mem_path)
                .map_err(|e| FrameworkError::ProcessError(
                    format!("Failed to open {}: {}", mem_path, e)
                ))?;
            
            let mut buffer = vec![0u8; size];
            let bytes_read = file.read_at(&mut buffer, address)
                .map_err(|e| FrameworkError::MemoryError(
                    format!("Failed to read memory at 0x{:x}: {}", address, e)
                ))?;
            
            buffer.truncate(bytes_read);
            
            if bytes_read == 0 {
                return Err(FrameworkError::MemoryError(
                    format!("No data read at address 0x{:x}", address)
                ));
            }
            
            Ok(buffer)
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Err(FrameworkError::MemoryError(
                "Process memory reading is not supported on this platform".to_string()
            ))
        }
    }
    
    fn write_bytes(&self, address: u64, data: &[u8]) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::FileExt;
            
            let mem_path = format!("/proc/{}/mem", self.pid);
            let file = File::options()
                .write(true)
                .open(&mem_path)
                .map_err(|e| FrameworkError::ProcessError(
                    format!("Failed to open {} for writing: {}", mem_path, e)
                ))?;
            
            let bytes_written = file.write_at(data, address)
                .map_err(|e| FrameworkError::MemoryError(
                    format!("Failed to write memory at 0x{:x}: {}", address, e)
                ))?;
            
            if bytes_written != data.len() {
                return Err(FrameworkError::MemoryError(
                    format!("Partial write at 0x{:x}: {} of {} bytes", 
                            address, bytes_written, data.len())
                ));
            }
            
            Ok(())
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Err(FrameworkError::MemoryError(
                "Process memory writing is not supported on this platform".to_string()
            ))
        }
    }
    
    fn get_memory_regions(&self) -> Result<Vec<MemoryRegion>> {
        Ok(self.regions.clone())
    }
    
    fn is_readable(&self, address: u64) -> bool {
        self.find_region(address)
            .map(|r| r.permissions.read)
            .unwrap_or(false)
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[cfg(target_os = "linux")]
fn parse_maps_line(line: &str) -> Option<MemoryRegion> {
    // Пример: 00400000-00452000 r-xp 00000000 08:02 173521      /usr/bin/cat
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return None;
    }
    
    let range: Vec<&str> = parts[0].split('-').collect();
    if range.len() != 2 {
        return None;
    }
    
    let start = u64::from_str_radix(range[0], 16).ok()?;
    let end = u64::from_str_radix(range[1], 16).ok()?;
    
    let perms = &parts[1];
    let permissions = Permissions::new(
        perms.contains('r'),
        perms.contains('w'),
        perms.contains('x'),
    );
    
    let name = if parts.len() > 5 {
        Some(parts[5..].join(" "))
    } else {
        None
    };
    
    Some(MemoryRegion {
        start,
        end,
        size: (end - start) as usize,
        permissions,
        name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_maps_line() {
        let line = "00400000-00452000 r-xp 00000000 08:02 173521 /usr/bin/cat";
        let region = parse_maps_line(line).unwrap();
        
        assert_eq!(region.start, 0x00400000);
        assert_eq!(region.end, 0x00452000);
        assert!(region.permissions.read);
        assert!(!region.permissions.write);
        assert!(region.permissions.execute);
        assert_eq!(region.name.as_deref(), Some("/usr/bin/cat"));
    }
    
    #[test]
    fn test_process_memory_new() {
        // Тест на текущем процессе
        let pid = std::process::id();
        let process = ProcessMemory::new(pid);
        assert!(process.is_ok());
    }
}
