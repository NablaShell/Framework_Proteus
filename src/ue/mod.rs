// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

pub mod fname;
pub mod uobject;
pub mod scanner;


/// Базовые константы UE4/5
pub const GNAMES_PATTERN: &str = "48 8B 05 ?? ?? ?? ?? 48 85 C0 75 ??";
pub const GOBJECTS_PATTERN: &str = "48 8B 05 ?? ?? ?? ?? 48 8B 0C C8 48 8D 04 D1";
pub const GWORLD_PATTERN: &str = "48 8B 1D ?? ?? ?? ?? 48 85 DB 74 ??";
pub const FNAME_PATTERN: &str = "48 8D 05 ?? ?? ?? ?? 48 89 44 24";

/// Версия UE
#[derive(Debug, Clone, PartialEq)]
pub enum UEVersion {
    UE4,
    UE5,
}

/// Основная структура UE контекста
#[derive(Debug)]
pub struct UEContext {
    pub version: UEVersion,
    pub gnames_addr: u64,
    pub gobjects_addr: u64,
    pub gworld_addr: u64,
    pub base_address: u64,
}

impl UEContext {
    pub fn new(version: UEVersion, base_address: u64) -> Self {
        Self {
            version,
            gnames_addr: 0,
            gobjects_addr: 0,
            gworld_addr: 0,
            base_address,
        }
    }
}
