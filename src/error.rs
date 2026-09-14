// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FrameworkError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Step execution error: {0}")]
    StepError(String),

    #[error("Context error: {0}")]
    ContextError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Lua error: {0}")]
    LuaError(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("Process error: {0}")]
    ProcessError(String),

    #[error("Scan error: {0}")]
    ScanError(String),
}

impl From<mlua::Error> for FrameworkError {
    fn from(err: mlua::Error) -> Self {
        FrameworkError::LuaError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, FrameworkError>;
