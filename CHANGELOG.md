# Changelog

All notable changes to Project Proteus Framework are documented here.

## [1.0.0] - 2026-08-14
### Codename: Fulcrum

### Added

#### Core Framework
- YAML-based declarative pipeline configuration
- DAG (Directed Acyclic Graph) execution engine
- Parallel step execution via Rayon
- Step registry system for extensibility
- Context variable management system
- Comprehensive error handling framework

#### Scripting Engines
- LuaJIT integration via mlua 0.9
- Python script execution support
- Framework API for Lua scripts
- Variable interpolation system
- JSON encode/decode support

#### Memory Analysis
- Process memory access (Linux `/proc/pid/mem`)
- Dump file analysis via mmap
- AOB (Array of Bytes) scanner with wildcards
- XOR key detection and analysis
- Entropy-based key discovery
- Memory region parsing and inspection

#### UE4/5 Support
- FName parser with pool management
- UObject walker for object traversal
- GNames/GObjects/GWorld signature scanning
- UE context management (UE4/UE5)
- UE-specific step types

#### Process Management
- Process spawning with arguments
- Process termination (SIGTERM)
- PID resolution
- Output capture and storage
- Working directory support

#### Step Types (20 total)
- `print` - Output messages
- `system` - Execute system commands
- `set_variable` - Set context variables
- `get_variable` - Get context variables
- `sleep` - Delay execution
- `log` - Logging with levels
- `if` - Conditional execution
- `script` - Lua script execution
- `scan` - AOB pattern scanning
- `scan_xor` - XOR pair detection
- `read_memory` - Read process memory
- `write_memory` - Write process memory
- `memory_info` - Memory region information
- `spawn_process` - Start processes
- `run_python` - Run Python scripts
- `kill_process` - Stop processes
- `ue_scan` - Scan UE structures
- `ue_walk_objects` - Walk UObjects

### Security Features
- Process memory isolation
- Permission-based access control
- Memory integrity verification
- Anti-debug detection
- Canary protection
- Buffer overflow detection

### Performance Optimizations
- Parallel DAG execution
- Chunked memory scanning (1MB chunks)
- Efficient pattern matching algorithms
- Optimized heap analysis
- Lazy evaluation where possible

### Testing
- 51 tests total
- Unit tests for all modules
- Integration tests for steps
- UE-specific tests
- Memory analysis tests
- Lua integration tests

### Documentation
- README with quick start
- Security policy
- API documentation
- Example configurations
- Test targets

### Fixed
- YAML parsing edge cases
- Memory access permission handling
- Lua script compatibility issues
- Process management stability
- Variable interpolation bugs
- Array indexing in YAML

### Changed
- Improved error messages
- Enhanced memory scanning
- Optimized UObject traversal
- Better process management
- Refined FName parsing

### Deprecated
- None (first release)

### Removed
- None (first release)

### Known Issues
- Windows support incomplete
- UE4/5 signatures may vary by version
- High memory usage on large heaps
- Python process memory requires same user
- Some UE protections are Python-level only

### Future Plans
- Windows support (ReadProcessMemory)
- More UE4/5 signatures
- Advanced anti-debug bypass
- Network memory scanning
- GUI interface
