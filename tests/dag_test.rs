// Copyright (C) 2026 NablaShell. All rights reserved.
// Licensed under the Functional Source License, Version 1.1, ALv2 Future License.
// See LICENSE file for details.

use framework_proteus::dag::DagBuilder;
use framework_proteus::{Config, PipelineExecutor};

#[test]
fn test_dag_creation() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "step1"
    name: "Step 1"
    action: "print"
    parameters:
      message: "One"

  - id: "step2"
    name: "Step 2"
    action: "print"
    parameters:
      message: "Two"
    depends_on: ["step1"]
"#;

    let config = Config::parse(content).unwrap();
    let mut dag = DagBuilder::new();
    dag.build(&config).unwrap();

    assert_eq!(dag.get_parallel_groups_count(), 2); // Два уровня
    assert_eq!(dag.get_execution_order()[0], vec!["step1"]);
    assert_eq!(dag.get_execution_order()[1], vec!["step2"]);
}

#[test]
fn test_parallel_execution_groups() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "a"
    name: "A"
    action: "print"
    parameters:
      message: "A"

  - id: "b"
    name: "B"
    action: "print"
    parameters:
      message: "B"

  - id: "c"
    name: "C"
    action: "print"
    parameters:
      message: "C"
    depends_on: ["a", "b"]
"#;

    let config = Config::parse(content).unwrap();
    let mut dag = DagBuilder::new();
    dag.build(&config).unwrap();

    // Два уровня: [a, b] и [c]
    assert_eq!(dag.get_parallel_groups_count(), 2);
    assert_eq!(dag.get_execution_order()[0].len(), 2);
    assert_eq!(dag.get_execution_order()[1], vec!["c"]);
}

#[test]
fn test_cyclic_dependency_detection() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "a"
    name: "A"
    action: "print"
    parameters:
      message: "A"
    depends_on: ["b"]

  - id: "b"
    name: "B"
    action: "print"
    parameters:
      message: "B"
    depends_on: ["a"]
"#;

    let result = Config::parse(content);
    assert!(result.is_err());

    match result {
        Err(e) => {
            assert!(e.to_string().contains("Cyclic dependency"));
        }
        Ok(_) => panic!("Should have detected cyclic dependency"),
    }
}

#[test]
fn test_execute_dag_pipeline() {
    let content = r#"
version: "1.0"
pipeline:
  - id: "init"
    name: "Initialize"
    action: "set_variable"
    parameters:
      name: "status"
      value: "ready"

  - id: "process"
    name: "Process"
    action: "set_variable"
    parameters:
      name: "result"
      value: "${status} processed"
    depends_on: ["init"]

  - id: "output"
    name: "Output"
    action: "print"
    parameters:
      message: "${result}"
    store_as: "final"
    depends_on: ["process"]
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();

    assert_eq!(
        context.get_variable("final").unwrap().to_string_value(),
        "ready processed"
    );
}

#[test]
fn test_conditional_execution() {
    let content = r#"
version: "1.0"
variables:
  debug: true

pipeline:
  - id: "check"
    name: "Check Condition"
    action: "if"
    parameters:
      condition: "${debug} == true"
    store_as: "condition_met"

  - id: "conditional_output"
    name: "Conditional Output"
    action: "print"
    parameters:
      message: "Debug is enabled"
    condition: "${condition_met} == true"
    depends_on: ["check"]
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();

    assert_eq!(
        context
            .get_variable("condition_met")
            .unwrap()
            .to_string_value(),
        "true"
    );
}

#[test]
fn test_complex_dag_with_conditions() {
    let content = r#"
version: "1.0"
variables:
  threshold: 50

pipeline:
  - id: "measure"
    name: "Measure"
    action: "set_variable"
    parameters:
      name: "value"
      value: 75

  - id: "evaluate"
    name: "Evaluate"
    action: "if"
    parameters:
      condition: "${value} > ${threshold}"
    depends_on: ["measure"]
    store_as: "above_threshold"

  - id: "action"
    name: "Action"
    action: "print"
    parameters:
      message: "Value is above threshold"
    condition: "${above_threshold} == true"
    depends_on: ["evaluate"]
    store_as: "action_taken"
"#;

    let config = Config::parse(content).unwrap();
    let executor = PipelineExecutor::new();
    let context = executor.execute(&config).unwrap();

    assert_eq!(
        context
            .get_variable("action_taken")
            .unwrap()
            .to_string_value(),
        "Value is above threshold"
    );
}
