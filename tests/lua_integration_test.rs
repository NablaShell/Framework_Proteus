// Copyright (C) 2026 Project Proteus Contributors. All rights reserved.
// Licensed under the Project Proteus Core Source-Available License Agreement.
// See LICENSE file for details.

use framework_proteus::{Config, PipelineExecutor};

#[test]
fn test_basic_lua_script() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "lua_test"
    name: "Lua Test"
    action: "script"
    parameters:
      script: |
        framework.set_var("test_value", 42)
        local value = framework.get_var("test_value")
        return value
    store_as: "result"
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();
    
    assert_eq!(
        context.get_variable("result").unwrap().to_string_value(),
        "42"
    );
}

#[test]
fn test_lua_with_variables() {
    let content = r#"
version: "1.0"
variables:
  name: "World"
pipeline:
  - id: "lua_greeting"
    name: "Lua Greeting"
    action: "script"
    parameters:
      script: |
        local name = framework.get_var("name")
        local greeting = "Hello, " .. name .. "!"
        framework.set_var("greeting", greeting)
        return greeting
    store_as: "result"
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();
    
    assert_eq!(
        context.get_variable("result").unwrap().to_string_value(),
        "Hello, World!"
    );
    assert_eq!(
        context.get_variable("greeting").unwrap().to_string_value(),
        "Hello, World!"
    );
}

#[test]
fn test_lua_script_file() {
    // Создаем временный файл скрипта
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(
        &mut temp_file,
        b"framework.set_var('from_file', true)\nreturn 'File script executed'"
    ).unwrap();
    
    let script_path = temp_file.path().to_str().unwrap();
    
    let content = format!(r#"
version: "1.0"
pipeline:
  - id: "file_script"
    name: "File Script"
    action: "script"
    parameters:
      script_path: "{}"
    store_as: "result"
"#, script_path);

    let config = Config::parse(&content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();
    
    assert_eq!(
        context.get_variable("result").unwrap().to_string_value(),
        "File script executed"
    );
    assert_eq!(
        context.get_variable("from_file").unwrap().to_string_value(),
        "true"
    );
}

#[test]
fn test_lua_error_handling() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "error_script"
    name: "Error Script"
    action: "script"
    parameters:
      script: |
        error("This is a test error")
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let result = executor.execute(&config);
    
    assert!(result.is_err());
    match result {
        Err(e) => {
            // Проверяем, что ошибка содержит информацию
            let error_str = e.to_string();
            println!("Full error: {}", error_str);
            // Проверяем что это ошибка pipeline
            assert!(error_str.contains("Pipeline failed"));
        }
        Ok(_) => panic!("Should have failed"),
    }
}

#[test]
fn test_lua_complex_data() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "complex_data"
    name: "Complex Data"
    action: "script"
    parameters:
      script: |
        local data = {
            name = "Test",
            values = {1, 2, 3, 4, 5},
            nested = {
                key = "value"
            }
        }
        
        local json = framework.json_encode(data)
        framework.set_var("json_data", json)
        
        local sum = 0
        for i, v in ipairs(data.values) do
            sum = sum + v
        end
        
        return {
            json = json,
            sum = sum,
            count = #data.values
        }
    store_as: "result"
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();
    
    let result = context.get_variable("result").unwrap();
    if let framework_proteus::config::Value::Object(obj) = result {
        assert_eq!(obj.get("sum").unwrap().to_string_value(), "15");
        assert_eq!(obj.get("count").unwrap().to_string_value(), "5");
    } else {
        panic!("Expected object result");
    }
    
    let json_data = context.get_variable("json_data").unwrap().to_string_value();
    assert!(json_data.contains("\"name\":\"Test\""));
    assert!(json_data.contains("\"values\":[1,2,3,4,5]"));
}
