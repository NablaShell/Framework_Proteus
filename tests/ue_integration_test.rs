// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use framework_proteus::{Config, PipelineExecutor};
use std::process::Command;

#[test]
fn test_ue_scan_with_python_process() {
    // Запускаем простой Python процесс
    let child = Command::new("python3")
        .arg("-c")
        .arg("import time; time.sleep(10)")
        .spawn()
        .unwrap();

    let pid = child.id();

    let content = format!(
        r#"
version: "1.0"
variables:
  target_pid: {}
pipeline:
  - id: "scan"
    name: "UE Scan"
    action: "ue_scan"
    parameters:
      pid: "${{target_pid}}"
"#,
        pid
    );

    let config = Config::parse(&content).unwrap();
    let executor = PipelineExecutor::new();
    let result = executor.execute(&config);

    // Убиваем процесс
    let _ = Command::new("kill").arg(pid.to_string()).status();

    // Python процесс доступен для чтения (тот же пользователь)
    assert!(result.is_ok());
}
