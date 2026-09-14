// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use crate::error::{FrameworkError, Result};
use crate::memory::{MemoryAccessor, MemoryRegion, ScanResult};
use rayon::prelude::*;

/// AOB (Array of Bytes) сканер
pub struct AobScanner {
    /// Паттерн с wildcards (0x?? или 0x? означает любой байт)
    pattern: Vec<Option<u8>>,
}

impl AobScanner {
    /// Создает сканер из строки паттерна
    /// Пример: "48 8B ?? ?? 8B 45 ??"
    pub fn new(pattern: &str) -> Result<Self> {
        let pattern = Self::parse_pattern(pattern)?;
        Ok(Self { pattern })
    }

    /// Создает сканер из массива байт с маской
    pub fn from_bytes(bytes: &[u8], mask: &[bool]) -> Result<Self> {
        if bytes.len() != mask.len() {
            return Err(FrameworkError::ScanError(
                "Pattern and mask must have the same length".to_string(),
            ));
        }

        let pattern: Vec<Option<u8>> = bytes
            .iter()
            .zip(mask.iter())
            .map(
                |(&byte, &is_known)| {
                    if is_known {
                        Some(byte)
                    } else {
                        None
                    }
                },
            )
            .collect();

        Ok(Self { pattern })
    }

    /// Возвращает длину паттерна
    pub fn len(&self) -> usize {
        self.pattern.len()
    }

    /// Проверяет, пуст ли паттерн
    pub fn is_empty(&self) -> bool {
        self.pattern.is_empty()
    }

    /// Возвращает паттерн в виде строки
    pub fn to_string_pattern(&self) -> String {
        self.pattern
            .iter()
            .map(|b| match b {
                Some(byte) => format!("{:02X}", byte),
                None => "??".to_string(),
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn parse_pattern(pattern: &str) -> Result<Vec<Option<u8>>> {
        let mut result = Vec::new();

        for token in pattern.split_whitespace() {
            if token == "??" || token == "?" {
                result.push(None);
            } else {
                let byte = u8::from_str_radix(token, 16).map_err(|e| {
                    FrameworkError::ScanError(format!("Invalid pattern byte '{}': {}", token, e))
                })?;
                result.push(Some(byte));
            }
        }

        if result.is_empty() {
            return Err(FrameworkError::ScanError("Empty pattern".to_string()));
        }

        Ok(result)
    }

    /// Сканирует память и возвращает все совпадения
    pub fn scan(&self, memory: &dyn MemoryAccessor) -> Result<Vec<ScanResult>> {
        let regions = memory.get_memory_regions()?;

        let results: Vec<ScanResult> = regions
            .par_iter()
            .filter(|region| region.permissions.read)
            .filter_map(|region| self.scan_region(memory, region).ok())
            .flatten()
            .collect();

        Ok(results)
    }

    /// Сканирует конкретный регион
    pub fn scan_region(
        &self,
        memory: &dyn MemoryAccessor,
        region: &MemoryRegion,
    ) -> Result<Vec<ScanResult>> {
        let mut results = Vec::new();

        // Читаем регион частями для экономии памяти
        let chunk_size = 1024 * 1024; // 1MB
        let mut offset = region.start;

        while offset < region.end {
            let size = std::cmp::min(chunk_size, (region.end - offset) as usize);

            if let Ok(data) = memory.read_bytes(offset, size) {
                let matches = self.find_matches(&data, offset);
                results.extend(matches);
            }

            offset += size as u64;
        }

        Ok(results)
    }

    /// Находит все совпадения паттерна в данных
    fn find_matches(&self, data: &[u8], base_address: u64) -> Vec<ScanResult> {
        let mut results = Vec::new();

        if data.len() < self.pattern.len() {
            return results;
        }

        let max_index = data.len() - self.pattern.len();

        'outer: for i in 0..=max_index {
            for (j, &pattern_byte) in self.pattern.iter().enumerate() {
                if let Some(expected) = pattern_byte {
                    if data[i + j] != expected {
                        continue 'outer;
                    }
                }
            }

            // Нашли совпадение
            results.push(ScanResult {
                address: base_address + i as u64,
                pattern: self.pattern.iter().map(|b| b.unwrap_or(0)).collect(),
                region: None,
            });
        }

        results
    }

    /// Сканирует и возвращает первое совпадение
    pub fn scan_first(&self, memory: &dyn MemoryAccessor) -> Result<Option<ScanResult>> {
        let results = self.scan(memory)?;
        Ok(results.into_iter().next())
    }

    /// Сканирует и возвращает все уникальные адреса
    pub fn scan_unique(&self, memory: &dyn MemoryAccessor) -> Result<Vec<u64>> {
        let results = self.scan(memory)?;
        let mut addresses: Vec<u64> = results.into_iter().map(|r| r.address).collect();

        addresses.sort_unstable();
        addresses.dedup();

        Ok(addresses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pattern() {
        let scanner = AobScanner::new("48 8B ?? ?? 8B 45 ??").unwrap();
        assert_eq!(scanner.len(), 7); // Исправлено с 6 на 7
        assert_eq!(scanner.pattern[0], Some(0x48));
        assert_eq!(scanner.pattern[1], Some(0x8B));
        assert_eq!(scanner.pattern[2], None);
        assert_eq!(scanner.pattern[3], None);
        assert_eq!(scanner.pattern[4], Some(0x8B));
        assert_eq!(scanner.pattern[5], Some(0x45));
        assert_eq!(scanner.pattern[6], None);
    }

    #[test]
    fn test_find_matches() {
        let scanner = AobScanner::new("48 8B ?? ??").unwrap();
        let data = vec![0x48, 0x8B, 0x11, 0x22, 0x48, 0x8B, 0x33, 0x44];

        let matches = scanner.find_matches(&data, 0x1000);

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].address, 0x1000);
        assert_eq!(matches[1].address, 0x1004);
    }

    #[test]
    fn test_no_matches() {
        let scanner = AobScanner::new("AA BB CC").unwrap();
        let data = vec![0x11, 0x22, 0x33, 0x44];

        let matches = scanner.find_matches(&data, 0);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_to_string_pattern() {
        let scanner = AobScanner::new("48 8B ?? ?? 8B").unwrap();
        assert_eq!(scanner.to_string_pattern(), "48 8B ?? ?? 8B");
    }
}
