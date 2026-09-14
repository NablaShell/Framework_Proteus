// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use framework_proteus::memory::scanner::AobScanner;
use framework_proteus::memory::dump::DumpMemory;
use framework_proteus::memory::MemoryAccessor;

#[test]
fn test_aob_scanner_pattern_parsing() {
    let scanner = AobScanner::new("48 8B ?? ?? 8B 45 ?? 90").unwrap();
    assert_eq!(scanner.len(), 8);
    assert_eq!(scanner.to_string_pattern(), "48 8B ?? ?? 8B 45 ?? 90");
}

#[test]
fn test_aob_scanner_find_matches() {
    let scanner = AobScanner::new("48 8B ?? ??").unwrap();
    let data = vec![0x48, 0x8B, 0x11, 0x22, 0x48, 0x8B, 0x33, 0x44];
    
    // Создаем тестовый дамп
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, &data).unwrap();
    
    let dump = DumpMemory::new(temp_file.path().to_str().unwrap(), 0x1000).unwrap();
    
    let results = scanner.scan(&dump).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].address, 0x1000);
    assert_eq!(results[1].address, 0x1004);
}

#[test]
fn test_dump_memory_read() {
    let data = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, &data).unwrap();
    
    let dump = DumpMemory::new(temp_file.path().to_str().unwrap(), 0x2000).unwrap();
    
    let bytes = dump.read_bytes(0x2000, 4).unwrap();
    assert_eq!(bytes, vec![0x11, 0x22, 0x33, 0x44]);
    
    let bytes = dump.read_bytes(0x2002, 2).unwrap();
    assert_eq!(bytes, vec![0x33, 0x44]);
}

#[test]
fn test_process_memory_regions() {
    use framework_proteus::memory::process::ProcessMemory;
    
    let pid = std::process::id();
    let process = ProcessMemory::new(pid).unwrap();
    let regions = process.get_memory_regions().unwrap();
    
    assert!(!regions.is_empty());
    
    // Проверяем, что есть хотя бы один читаемый регион
    assert!(regions.iter().any(|r| r.permissions.read));
}
