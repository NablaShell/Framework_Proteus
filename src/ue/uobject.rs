// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use crate::error::Result;
use crate::memory::MemoryAccessor;

/// UObject базовая структура
#[derive(Debug, Clone)]
pub struct UObject {
    pub vtable: u64,
    pub flags: u32,
    pub index: u32,
    pub class_private: u64,
    pub name_private: u64,
    pub outer_private: u64,
    pub name: String,
    pub class_name: String,
}

/// Обходчик UObject
pub struct UObjectWalker {
    pub objects: Vec<UObject>,
}

impl UObjectWalker {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    /// Обход UObject массива
    pub fn walk(
        &mut self,
        memory: &dyn MemoryAccessor,
        objects_base: u64,
        count: usize,
    ) -> Result<()> {
        // UE4 FUObjectItem: 24 bytes (0x18)
        // struct FUObjectItem {
        //     UObject* Object;      // 8 bytes
        //     int32_t Flags;        // 4 bytes
        //     int32_t ClusterIndex; // 4 bytes
        //     int32_t SerialNumber; // 4 bytes
        //     int32_t Reserved;     // 4 bytes
        // };
        let item_size = 24usize;

        for i in 0..count {
            let obj_addr = objects_base + (i * item_size) as u64;

            if let Ok(data) = memory.read_bytes(obj_addr, item_size) {
                if data.len() < item_size {
                    continue;
                }

                let obj_ptr = u64::from_le_bytes(data[0..8].try_into().unwrap_or([0; 8]));

                if obj_ptr == 0 {
                    continue;
                }

                // Читаем UObject
                if let Ok(obj_data) = memory.read_bytes(obj_ptr, 64) {
                    if obj_data.len() >= 64 {
                        let vtable =
                            u64::from_le_bytes(obj_data[0..8].try_into().unwrap_or([0; 8]));
                        let flags =
                            u32::from_le_bytes(obj_data[8..12].try_into().unwrap_or([0; 4]));
                        let index =
                            u32::from_le_bytes(obj_data[12..16].try_into().unwrap_or([0; 4]));
                        let class_private =
                            u64::from_le_bytes(obj_data[16..24].try_into().unwrap_or([0; 8]));
                        let name_private =
                            u64::from_le_bytes(obj_data[24..32].try_into().unwrap_or([0; 8]));
                        let outer_private =
                            u64::from_le_bytes(obj_data[32..40].try_into().unwrap_or([0; 8]));

                        self.objects.push(UObject {
                            vtable,
                            flags,
                            index,
                            class_private,
                            name_private,
                            outer_private,
                            name: format!("Object_{}", index),
                            class_name: format!("Class_{}", class_private),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Получение объекта по индексу
    pub fn get_object(&self, index: u32) -> Option<&UObject> {
        self.objects.iter().find(|obj| obj.index == index)
    }

    /// Получение количества объектов
    pub fn count(&self) -> usize {
        self.objects.len()
    }
}

impl Default for UObjectWalker {
    fn default() -> Self {
        Self::new()
    }
}
