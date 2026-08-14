# Proteus Framework

**Version:** 1.0.0 "Fulcrum"  
**License:** Project Proteus Core Source-Available License Agreement v1.0  
**Status:** Stable Release

## Overview

Framework Proteus is a declarative analysis framework designed for low-level memory inspection, process auditing, and binary analysis. It features a robust YAML-driven pipeline architecture with embedded LuaJIT scripting support, making it an ideal instrument for analyzing complex software environments, custom runtimes, and proprietary application structures.

## Core Features

### Declarative Pipeline
- YAML-based configuration
- DAG execution with parallel processing
- Dependency management
- Conditional execution
- Variable interpolation

### Memory Analysis
- AOB (Array of Bytes) pattern scanning
- XOR key detection and analysis
- Entropy-based key discovery
- Heap/stack region analysis
- Process memory access (Linux `/proc/pid/mem`)
- Memory dump file analysis

### UE4/5 Support
- FName parser with pool management
- UObject walker for object traversal
- GNames/GObjects/GWorld signature scanning
- UE context management (UE4/UE5)

### Scripting Engines
- LuaJIT integration via mlua
- Python script execution
- Framework API for both languages
- Variable interpolation in scripts

### Process Management
- Process spawning with argument support
- Process termination
- PID resolution by name
- Output capture and storage

## Installation

```bash
# Clone repository
git clone https://github.com/NablaShell/Framework_Proteus.git
cd Framework_Proteus

# Build
cargo build --release

# Run tests
cargo test
```

## Quick Start

```bash
# Run with configuration
cargo run -- configs/example.yaml

# Or use release binary
./target/release/framework-proteus configs/example.yaml
```

## Example Configuration

```yaml
version: "1.0"
target:
  name: "MyTarget"
  mode: "live"

variables:
  target_path: "/path/to/target"
  pattern: "48 8B ?? ?? 8B 45 ??"

pipeline:
  - id: "start_target"
    action: "spawn_process"
    parameters:
      command: "${target_path}"
      wait: false
    store_as: "pid"

  - id: "wait"
    action: "sleep"
    parameters:
      duration_ms: 2000
    depends_on: ["start_target"]

  - id: "scan"
    action: "scan"
    parameters:
      pid: "${pid}"
      pattern: "${pattern}"
    store_as: "addresses"
    depends_on: ["wait"]

  - id: "analyze"
    action: "script"
    parameters:
      script: |
        local addresses = framework.get_variable("addresses")
        framework.print("Found " .. #addresses .. " matches")
    depends_on: ["scan"]

  - id: "cleanup"
    action: "kill_process"
    parameters:
      pid: "${pid}"
    depends_on: ["analyze"]
```

## Step Types

| Step | Description | Parameters |
|------|-------------|------------|
| `print` | Output message | `message` |
| `system` | Execute command | `command`, `use_shell` |
| `set_variable` | Set context variable | `name`, `value` |
| `get_variable` | Get context variable | `name` |
| `sleep` | Delay execution | `duration_ms` |
| `log` | Log message | `level`, `message` |
| `if` | Conditional check | `condition` |
| `script` | Run Lua script | `script` or `script_path` |
| `scan` | AOB scan | `pattern`, `pid` |
| `scan_xor` | XOR pair scan | `target_value`, `pid` |
| `read_memory` | Read process memory | `address`, `size`, `pid` |
| `write_memory` | Write process memory | `address`, `data`, `pid` |
| `memory_info` | Memory region info | `pid`, `max_display` |
| `spawn_process` | Start process | `command`, `args`, `wait` |
| `run_python` | Run Python script | `script` or `script_path` |
| `kill_process` | Stop process | `pid` |
| `ue_scan` | Scan UE structures | `pid` |
| `ue_walk_objects` | Walk UObjects | `pid`, `max_objects` |

## Framework API

### Lua API

```lua
-- Variables
framework.set_variable(name, value)
framework.get_variable(name)
framework.has_variable(name)

-- Logging
framework.print(message)
framework.log(level, message)

-- System
framework.execute_command(command)
framework.sleep(milliseconds)

-- JSON
framework.json_encode(value)
framework.json_decode(json_string)

-- Time
framework.get_timestamp()
```

### Python API

```python
import os
import sys

# Process info
pid = os.getpid()
print(f"PID: {pid}")

# Memory access
with open(f"/proc/{pid}/maps", "r") as f:
    maps = f.read()
```

## Memory Analysis

### AOB Scanning

```yaml
- id: "aob_scan"
  action: "scan"
  parameters:
    pattern: "48 8B ?? ?? 8B 45 ??"  # Wildcards supported
    max_display: 10
```

### XOR Analysis

```yaml
- id: "xor_scan"
  action: "scan_xor"
  parameters:
    target_value: 100  # Find pairs where a XOR b = 100
```

### Entropy Analysis

```python
# Find high-entropy keys (AES etc.)
import math

def entropy(data):
    freq = {}
    for byte in data:
        freq[byte] = freq.get(byte, 0) + 1
    ent = 0
    for count in freq.values():
        p = count / len(data)
        ent -= p * math.log2(p)
    return ent
```

## UE4/5 Analysis

```yaml
pipeline:
  - id: "ue_scan"
    action: "ue_scan"
    parameters:
      pid: "${pid}"

  - id: "ue_walk"
    action: "ue_walk_objects"
    parameters:
      pid: "${pid}"
      max_objects: 1000
    depends_on: ["ue_scan"]
```

## Project Structure

```
framework/
├── src/
│   ├── config.rs           # YAML configuration
│   ├── context.rs          # Execution context
│   ├── error.rs            # Error handling
│   ├── executor.rs         # Pipeline executor
│   ├── pipeline.rs         # Pipeline types
│   ├── steps/              # Step implementations
│   │   ├── print.rs
│   │   ├── system.rs
│   │   ├── variables.rs
│   │   ├── control.rs
│   │   ├── condition.rs
│   │   ├── script.rs       # Lua execution
│   │   ├── process.rs      # Process management
│   │   ├── scan.rs         # AOB/XOR scanning
│   │   ├── memory.rs       # Memory operations
│   │   └── ue_steps.rs     # UE steps
│   ├── memory/             # Memory accessors
│   │   ├── process.rs      # Process memory
│   │   ├── dump.rs         # Dump files
│   │   └── scanner.rs      # Pattern scanner
│   ├── luajit_engine/      # LuaJIT integration
│   │   ├── api.rs          # Framework API
│   │   └── registry.rs     # Function registration
│   ├── ue/                 # UE4/5 support
│   │   ├── fname.rs        # FName parser
│   │   ├── uobject.rs      # UObject walker
│   │   └── scanner.rs      # UE signatures
│   └── dag/                # DAG execution
│       ├── builder.rs
│       ├── scheduler.rs
│       └── validator.rs
├── configs/                # Example configs
├── targets/                # Test targets
└── tests/                  # Integration tests
```

## Testing

```bash
# All tests
cargo test

# Specific test
cargo test --test ue_test

# With output
cargo test -- --nocapture
```

## Performance

- Parallel DAG execution via Rayon
- Chunked memory scanning (1MB chunks)
- Efficient pattern matching
- Optimized heap analysis

## Contributing

1. Fork repository
2. Create feature branch
3. Commit changes
4. Push to branch
5. Create Pull Request

## License

This project is licensed under the Project Proteus Core Source-Available License Agreement v1.0.

## Contact

- Issues: GitHub Issues
- Security: nablashell@gmail.com
