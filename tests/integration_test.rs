// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use framework_proteus::{Config, PipelineExecutor};

#[test]
fn test_load_valid_config() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "test"
    name: "Test Step"
    action: "print"
    parameters:
      message: "Hello"
"#;

    let config = Config::parse(content).unwrap();
    assert_eq!(config.version, "1.0");
    assert_eq!(config.pipeline.len(), 1);
    assert_eq!(config.pipeline[0].id, "test");
}

#[test]
fn test_load_invalid_version() {
    let content = r#"
version: "2.0"
pipeline: []
"#;

    let result = Config::parse(content);
    assert!(result.is_err());
}

#[test]
fn test_duplicate_step_ids() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "same"
    name: "Step 1"
    action: "print"
    parameters:
      message: "First"
  - id: "same"
    name: "Step 2"
    action: "print"
    parameters:
      message: "Second"
"#;

    let result = Config::parse(content);
    assert!(result.is_err());
}

#[test]
fn test_execute_print_pipeline() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "hello"
    name: "Hello World"
    action: "print"
    parameters:
      message: "Hello, World!"
    store_as: "greeting"
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();

    // Проверяем, что переменная сохранилась через store_as
    assert_eq!(
        context.get_variable("greeting").unwrap().to_string_value(),
        "Hello, World!"
    );
}

#[test]
fn test_execute_variable_pipeline() {
    let content = r#"
version: "1.0"
variables:
  name: "Test"
pipeline:
  - id: "set_var"
    name: "Set Variable"
    action: "set_variable"
    parameters:
      name: "greeting"
      value: "Hello, ${name}!"
    store_as: "result"
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();

    assert_eq!(
        context.get_variable("greeting").unwrap().to_string_value(),
        "Hello, Test!"
    );
    assert_eq!(
        context.get_variable("result").unwrap().to_string_value(),
        "Hello, Test!"
    );
}

#[test]
fn test_execute_system_command() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "echo"
    name: "Echo Test"
    action: "system"
    parameters:
      command: "echo test123"
    store_as: "echo_result"
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();

    let result = context
        .get_variable("echo_result")
        .unwrap()
        .to_string_value();
    assert!(result.contains("test123"));
}

#[test]
fn test_execute_missing_executor() {
    // Теперь проверяем, что YAML с неизвестным действием не парсится
    let content = r#"
version: "1.0"
pipeline:
  - id: "unknown"
    name: "Unknown Action"
    action: "non_existent_action"
    parameters: {}
"#;

    // Конфигурация не должна парситься из-за неизвестного действия
    let result = Config::parse(content);
    assert!(result.is_err());

    // Проверяем, что ошибка содержит информацию о неизвестном действии
    match result {
        Err(e) => {
            assert!(e.to_string().contains("unknown variant"));
        }
        Ok(_) => panic!("Should have failed to parse unknown action"),
    }
}

#[test]
fn test_context_interpolation() {
    let mut context = framework_proteus::context::ExecutionContext::new();
    context.set_variable(
        "name",
        framework_proteus::config::Value::String("World".to_string()),
    );

    let result = context.interpolate_string("Hello, ${name}!");
    assert_eq!(result, "Hello, World!");
}

#[test]
fn test_complex_pipeline() {
    let content = r#"
version: "1.0"
variables:
  prefix: "Test"
pipeline:
  - id: "init"
    name: "Initialize"
    action: "set_variable"
    parameters:
      name: "count"
      value: 5

  - id: "message"
    name: "Create Message"
    action: "set_variable"
    parameters:
      name: "msg"
      value: "${prefix} count: ${count}"

  - id: "output"
    name: "Output"
    action: "print"
    parameters:
      message: "${msg}"
    store_as: "final_output"
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();

    assert_eq!(
        context
            .get_variable("final_output")
            .unwrap()
            .to_string_value(),
        "Test count: 5"
    );
}

// Дополнительные тесты для проверки различных сценариев

#[test]
fn test_execute_multiple_steps() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "step1"
    name: "Step 1"
    action: "set_variable"
    parameters:
      name: "value1"
      value: "First"

  - id: "step2"
    name: "Step 2"
    action: "set_variable"
    parameters:
      name: "value2"
      value: "Second"

  - id: "step3"
    name: "Step 3"
    action: "print"
    parameters:
      message: "${value1} and ${value2}"
    store_as: "combined"
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();

    assert_eq!(
        context.get_variable("combined").unwrap().to_string_value(),
        "First and Second"
    );
}

#[test]
fn test_execute_with_global_variables() {
    let content = r#"
version: "1.0"
variables:
  app_name: "TestApp"
  version: "2.0"

pipeline:
  - id: "info"
    name: "Show Info"
    action: "print"
    parameters:
      message: "${app_name} v${version}"
    store_as: "app_info"
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();

    assert_eq!(
        context.get_variable("app_info").unwrap().to_string_value(),
        "TestApp v2.0"
    );
}

#[test]
fn test_execute_sleep_step() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "pause"
    name: "Pause"
    action: "sleep"
    parameters:
      duration_ms: 10
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let start = std::time::Instant::now();
    let _context = executor.execute(&config).unwrap();
    let duration = start.elapsed();

    // Проверяем, что шаг sleep действительно ждал
    assert!(duration.as_millis() >= 10);
}

#[test]
fn test_error_on_missing_parameter() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "bad_print"
    name: "Bad Print"
    action: "print"
    parameters: {}
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let result = executor.execute(&config);

    assert!(result.is_err());
}
