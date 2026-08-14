// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use framework_proteus::ue::fname::{FNameParser, FNamePool};
use framework_proteus::ue::scanner::UEScanner;
use framework_proteus::ue::uobject::UObjectWalker;
use framework_proteus::ue::{UEContext, UEVersion, GNAMES_PATTERN, GOBJECTS_PATTERN, GWORLD_PATTERN};
use framework_proteus::memory::scanner::AobScanner;
use framework_proteus::memory::dump::DumpMemory;

#[test]
fn test_ue_context_creation() {
    let ctx = UEContext::new(UEVersion::UE5, 0x400000);
    assert_eq!(ctx.version, UEVersion::UE5);
    assert_eq!(ctx.base_address, 0x400000);
    assert_eq!(ctx.gnames_addr, 0);
    assert_eq!(ctx.gobjects_addr, 0);
    assert_eq!(ctx.gworld_addr, 0);
}

#[test]
fn test_fname_pool_basic() {
    let mut pool = FNamePool::new(0x1000);
    pool.add_name(0, "None".to_string());
    pool.add_name(1, "Core".to_string());
    pool.add_name(2, "Engine".to_string());
    
    assert_eq!(pool.get_name(0), Some("None"));
    assert_eq!(pool.get_name(1), Some("Core"));
    assert_eq!(pool.get_name(2), Some("Engine"));
    assert_eq!(pool.get_name(3), None);
}

#[test]
fn test_fname_parser() {
    let parser = FNameParser::new();
    
    // Создаем тестовые данные FName
    let data = vec![
        0x01, 0x00, 0x00, 0x00,  // Index = 1
        0x00, 0x00, 0x00, 0x00,  // Number = 0
    ];
    
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, &data).unwrap();
    
    let dump = DumpMemory::new(temp_file.path().to_str().unwrap(), 0x2000).unwrap();
    
    let fname = parser.parse_fname(&dump, 0x2000).unwrap();
    assert_eq!(fname.index, 1);
    assert_eq!(fname.number, 0);
}

#[test]
fn test_fname_string_parsing() {
    let parser = FNameParser::new();
    
    // Создаем тестовую строку с нулевым терминатором
    let mut data = b"PlayerController\0".to_vec();
    data.extend_from_slice(&[0u8; 16]); // Дополняем до 32 байт
    
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, &data).unwrap();
    
    let dump = DumpMemory::new(temp_file.path().to_str().unwrap(), 0x3000).unwrap();
    
    let name = parser.parse_fname_string(&dump, 0x3000).unwrap();
    assert_eq!(name, "PlayerController");
}

#[test]
fn test_uobject_walker_empty() {
    let walker = UObjectWalker::new();
    assert_eq!(walker.count(), 0);
    assert!(walker.get_object(0).is_none());
}

#[test]
fn test_uobject_walker_basic() {
    let mut walker = UObjectWalker::new();
    
    // Создаем непрерывный дамп
    let mut data = Vec::new();
    
    // Адрес 0x4000-0x4018: FUObjectItem[0] - пустой (24 байта)
    data.extend_from_slice(&[0u8; 24]);
    
    // Адрес 0x4018-0x4030: FUObjectItem[1] - валидный (указывает на 0x4030)
    data.extend_from_slice(&(0x4030u64).to_le_bytes());  // Object ptr -> 0x4030
    data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);  // Flags
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);  // ClusterIndex
    data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);  // SerialNumber
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);  // Reserved
    
    // Адрес 0x4030-0x4070: UObject данные (64 байта)
    data.extend_from_slice(&(0x5000u64).to_le_bytes());  // VTable
    data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);  // Flags
    data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);  // Index = 1
    data.extend_from_slice(&[0u8; 52]);  // Padding до 64 байт
    
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, &data).unwrap();
    
    let dump = DumpMemory::new(temp_file.path().to_str().unwrap(), 0x4000).unwrap();
    
    walker.walk(&dump, 0x4000, 2).unwrap();
    assert_eq!(walker.count(), 1);
}

#[test]
fn test_ue_scanner_patterns() {
    // Проверяем, что паттерны валидны
    let scanner1 = AobScanner::new(GNAMES_PATTERN).unwrap();
    let scanner2 = AobScanner::new(GOBJECTS_PATTERN).unwrap();
    let scanner3 = AobScanner::new(GWORLD_PATTERN).unwrap();
    
    assert!(scanner1.len() > 0);
    assert!(scanner2.len() > 0);
    assert!(scanner3.len() > 0);
}

#[test]
fn test_ue_scanner_no_match() {
    let scanner = UEScanner::new();
    
    // Пустой дамп
    let data = vec![0u8; 1024];
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, &data).unwrap();
    
    let dump = DumpMemory::new(temp_file.path().to_str().unwrap(), 0x5000).unwrap();
    
    // Сканер должен вернуть ошибку (не найти паттерн)
    let result = scanner.find_gnames(&dump);
    assert!(result.is_err());
}

#[test]
fn test_ue_version_comparison() {
    let ue4 = UEVersion::UE4;
    let ue5 = UEVersion::UE5;
    
    assert_ne!(ue4, ue5);
    assert_eq!(ue4, UEVersion::UE4);
    assert_eq!(ue5, UEVersion::UE5);
}

#[test]
fn test_fname_pool_multiple() {
    let mut pool = FNamePool::new(0x6000);
    
    // Добавляем много имен
    for i in 0..100 {
        pool.add_name(i, format!("Name_{}", i));
    }
    
    assert_eq!(pool.get_name(0), Some("Name_0"));
    assert_eq!(pool.get_name(50), Some("Name_50"));
    assert_eq!(pool.get_name(99), Some("Name_99"));
    assert_eq!(pool.get_name(100), None);
}

#[test]
fn test_uobject_walker_with_memory() {
    let mut walker = UObjectWalker::new();
    
    // Создаем непрерывный дамп
    let mut data = Vec::new();
    
    let objects_base = 0x6000u64;
    
    // FUObjectItem массив (10 элементов по 24 байта = 240 байт)
    // Указатели указывают на адреса сразу после массива
    let objects_data_base = objects_base + 240; // 0x60F0
    
    for i in 0..10u64 {
        let obj_ptr = objects_data_base + i * 64;
        data.extend_from_slice(&obj_ptr.to_le_bytes());  // Object ptr
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);  // Flags
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);  // ClusterIndex
        data.extend_from_slice(&[i as u8, 0x00, 0x00, 0x00]);  // SerialNumber
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);  // Reserved
    }
    
    // Данные объектов (10 объектов по 64 байта = 640 байт)
    for i in 0..10u64 {
        data.extend_from_slice(&(0x8000u64 + i * 8).to_le_bytes());  // VTable
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);  // Flags
        data.extend_from_slice(&[i as u8, 0x00, 0x00, 0x00]);  // Index
        data.extend_from_slice(&[0u8; 48]);  // Padding до 64 байт
    }
    
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, &data).unwrap();
    
    let dump = DumpMemory::new(temp_file.path().to_str().unwrap(), objects_base).unwrap();
    
    walker.walk(&dump, objects_base, 10).unwrap();
    assert_eq!(walker.count(), 10);
}
