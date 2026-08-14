// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use framework_proteus::Config;

#[test]
fn test_valid_complex_config() {
    let content = r#"
version: "1.0"
target:
  name: "TestTarget"
  mode: "live"
  parameters:
    pid: "1234"

variables:
  debug: true
  threshold: 100

pipeline:
  - id: "init"
    name: "Initialize"
    action: "set_variable"
    parameters:
      name: "status"
      value: "initialized"
    store_as: "init_status"

  - id: "check"
    name: "Check Status"
    action: "print"
    parameters:
      message: "Status: ${status}"
    depends_on: ["init"]
"#;

    let config = Config::parse(content).unwrap();

    assert_eq!(config.version, "1.0");
    assert_eq!(config.pipeline.len(), 2);
    assert_eq!(config.pipeline[1].depends_on, vec!["init"]);

    if let Some(target) = &config.target {
        assert_eq!(target.name, "TestTarget");
        assert_eq!(target.parameters.get("pid").unwrap(), "1234");
    } else {
        panic!("Target should be present");
    }
}

#[test]
fn test_invalid_dependency() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "step1"
    name: "Step 1"
    action: "print"
    parameters:
      message: "Hello"
    depends_on: ["non_existent_step"]
"#;

    let result = Config::parse(content);
    assert!(result.is_err());

    match result {
        Err(e) => {
            assert!(e.to_string().contains("depends on non-existent step"));
        }
        Ok(_) => panic!("Should have failed validation"),
    }
}

#[test]
fn test_empty_pipeline() {
    let content = r#"
version: "1.0"
pipeline: []
"#;

    let config = Config::parse(content).unwrap();
    assert_eq!(config.pipeline.len(), 0);
}

#[test]
fn test_missing_version() {
    let content = r#"
pipeline:
  - id: "test"
    name: "Test"
    action: "print"
    parameters:
      message: "Hello"
"#;

    let result = Config::parse(content);
    assert!(result.is_err());
}

#[test]
fn test_target_modes() {
    // Test live mode
    let live_config = r#"
version: "1.0"
target:
  name: "Process"
  mode: "live"
pipeline: []
"#;
    let config = Config::parse(live_config).unwrap();
    match config.target.unwrap().mode {
        framework_proteus::config::TargetMode::Live => {}
        _ => panic!("Should be Live mode"),
    }

    // Test dump mode
    let dump_config = r#"
version: "1.0"
target:
  name: "Dump"
  mode: "dump"
pipeline: []
"#;
    let config = Config::parse(dump_config).unwrap();
    match config.target.unwrap().mode {
        framework_proteus::config::TargetMode::Dump => {}
        _ => panic!("Should be Dump mode"),
    }

    // Test file mode
    let file_config = r#"
version: "1.0"
target:
  name: "File"
  mode: "file"
pipeline: []
"#;
    let config = Config::parse(file_config).unwrap();
    match config.target.unwrap().mode {
        framework_proteus::config::TargetMode::File => {}
        _ => panic!("Should be File mode"),
    }
}
