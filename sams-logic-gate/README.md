# SAMS Logic-Gate

**The Brain** - A real-time middleware processor for SAMS Atoms

## Overview

SAMS Logic-Gate acts as an intelligent middleware processor between Ghost-Node and Cyber-Monitor, implementing real-time data transformation and rule-based processing of SAMS Atoms.

## Architecture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ GHOST-NODE  │───▶│ LOGIC-GATE  │───▶│ DASHBOARD   │
└─────────────┘    └─────────────┘    └─────────────┘
```

## Features

### SAMS Semantic Logic Controller v0.1.0

**Professional Middleware with Hybrid Open-Source/Proprietary Architecture**

## Overview

SAMS Semantic Logic Controller (SLC) is a professional-grade middleware processor that implements advanced semantic reasoning, stateful logic, and intelligent intervention capabilities for SAMS (Semantic Atom Monitoring System) data streams. It features a hybrid architecture combining open-source transport with proprietary intelligence.

## Features

### 🧠 Core Functionality
- **Semantic Processing**: Advanced 32-byte atom analysis
- **Stateful Logic**: System health tracking and trend analysis
- **Intelligent Intervention**: Automatic high-energy sequence detection
- **Input Validation**: Anti-replay protection and integrity checks
- **Hybrid Architecture**: Open-source UI with proprietary reasoning

### ⚡ Performance Characteristics
- **Processing Latency**: < 1μs average logic execution
- **High Throughput**: 10,000+ atoms/second capability
- **Memory Efficiency**: < 50MB runtime footprint
- **Network Performance**: Sub-100μs end-to-end latency

### 🛡️ Security & Intelligence
- **Pattern Recognition**: Advanced semantic analysis algorithms
- **Intervention Logic**: Intelligent system protection
- **Anti-Replay**: Duplicate packet prevention
- **State Machine**: Multi-level health monitoring

## Architecture Overview

### Hybrid Model
```
┌─────────────────────────────────────────────────────────┐
│                OPEN-SOURCE LAYER                  │
├─────────────────────────────────────────────────────────┤
│ UDP Transport │ Terminal UI │ Integration API │ Documentation │
│ (Port 5555) │ (Cyan Theme) │ (Generic Trait) │ (MIT License) │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              PROPRIETARY LAYER                   │
├─────────────────────────────────────────────────────────┤
│ Semantic Engine │ Pattern Recog │ Intervention │ Decision Mat │
│ (Advanced AI) │ (Heuristics) │ (Protection) │ (Trade Secret) │
└─────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites
- Rust 1.70+ with tokio runtime
- Terminal with UTF-8 support
- Network access for UDP communication

### Installation & Running
```bash
# Clone the repository
git clone <repository-url>
cd sams-logic-gate

# Open-source mode (default)
cargo build --release
cargo run --release

# Closed-source mode (commercial)
cargo build --release --features "closed-source"
cargo run --release --features "closed-source"
```

### Development Modes
```bash
# Community development
cargo build --features "open-source"

# Commercial deployment
cargo build --features "closed-source"

# Feature verification
cargo check --features "open-source"    # Should compile
cargo check --features "closed-source"   # Should compile with proprietary libs
```

## Data Processing

### Semantic Analysis
```rust
// Input atom structure
pub struct SemanticAtom {
    pub id: String,           // Identifier
    pub timestamp: u64,        // Unix epoch
    pub energy_cost: f64,       // μJ (microjoules)
    pub trust_pqc: bool,        // Quantum cryptography trust
    pub data: HashMap<...>,      // Extensible metadata
    pub tags: Vec<String>,        // Processing annotations
    pub payload: Option<Vec<u8>>, // Raw payload data
}

// Processing result
pub struct ProcessedSemanticAtom {
    pub original: SemanticAtom,           // Input atom
    pub processing_time: Duration,        // Execution time
    pub tags_added: Vec<String>,          // New annotations
    pub security_alert: Option<String>,     // Security issues
    pub intervention_applied: bool,          // Protection triggered
    pub system_health: SystemHealth,        // Current state
}
```

### System Health States
- **🟢 Optimal**: Normal operation, all systems healthy
- **🟡 Warning**: Elevated activity or minor issues detected
- **🔴 Critical**: Multiple concerning patterns present
- **🟣 Intervention**: Automatic protection activated

### Intelligence Features
- **High Load Detection**: Energy cost > 100μJ analysis
- **Security Analysis**: PQC trust failure detection
- **Pattern Recognition**: Anomaly and trend detection
- **Intervention Logic**: 3+ high-energy sequence protection

## Configuration

### Open-Source Development
```toml
[development]
mode = "open-source"              # Use mock logic
enable_debug = true               # Development features
mock_data_file = "test_atoms.json" # Test data source
performance_profiling = true         # Enable metrics
```

### Commercial Deployment
```toml
[deployment]
mode = "closed-source"            # Use proprietary logic
license_key_file = "license.key"    # Commercial license
proprietary_lib_path = "proprietary_blobs/"  # Binary libraries
enable_optimizations = true         # Performance features
```

### Processing Parameters
```toml
[logic]
energy_threshold_uj = 100.0      # High load threshold
intervention_sequence = 3           # Trigger condition
intervention_cooldown_s = 5        # Protection interval
replay_window_ms = 1000           # Anti-replay window
health_analysis_window = 10          # System health atoms
```

## Performance Monitoring

### Real-Time Metrics
- **Logic Execution**: Sub-microsecond processing time
- **System Health**: Current operational state
- **Intervention Count**: Automatic protections applied
- **Processing Rate**: Atoms/second throughput
- **Memory Usage**: Runtime resource consumption

### Intelligence Analytics
- **Pattern Detection**: Anomaly recognition results
- **Trend Analysis**: Long-term system behavior
- **Efficiency Metrics**: Processing optimization insights
- **Security Events**: PQC trust and integrity issues

## Integration

### SAMS Ecosystem
```
Ghost-Node → SLC → Cyber-Monitor
   (5555)   (5555→5556)   (5556)
                    ↓
                Black-Box (Audit)
                (5556)
```

### API Interface
```rust
// Generic logic controller trait
pub trait LogicController {
    fn process_atom(&self, atom: SemanticAtom) -> Option<ProcessedSemanticAtom>;
}

// Factory function for both implementations
pub fn create_logic_controller() -> Box<dyn LogicController + Send + Sync>;
```

### Data Flow
1. **Input**: UDP packet reception from Ghost-Node
2. **Validation**: Anti-replay and integrity checks
3. **Processing**: Semantic analysis and state updates
4. **Intervention**: Intelligent protection when needed
5. **Output**: Forward processed atoms to Cyber-Monitor

## Development

### Open-Source Contributions
- **UI Layer**: Terminal interface improvements
- **Network Layer**: Transport optimization
- **Integration**: API enhancements
- **Documentation**: Guides and examples
- **Testing**: Validation and performance tests

### Proprietary Components
- **Core Algorithms**: Advanced semantic reasoning
- **Pattern Recognition**: Proprietary heuristics
- **Decision Matrices**: Intervention logic
- **Performance Optimizations**: Speed-critical code

### Build System
```bash
# Development build (open-source)
cargo build --features "open-source"

# Production build (closed-source)
cargo build --features "closed-source"

# Cross-compilation
cargo build --target x86_64-unknown-linux-musl

# Release optimization
cargo build --release --lto=fat
```

## IP Protection

### Code Structure
```
src/
├── common_types.rs          # Shared types and traits
├── logic_controller.rs     # Factory abstraction
├── main.rs                 # Open-source orchestration
├── mock_logic.rs          # Open-source implementation
├── slc_core.rs            # PROPRIETARY - Protected IP
└── LICENSE_PROPRIETARY.md  # Commercial license terms
```

### Repository Protection
- **Git Ignore**: `proprietary_blobs/` excluded
- **Build Guards**: Feature-based conditional compilation
- **License Headers**: Comprehensive IP notices
- **Distribution**: Separate open-source and commercial versions

## Licensing

### Open-Source Components (MIT License)
- Transport protocol and networking
- Terminal UI framework
- Mock implementation for development
- Integration documentation
- Build system and configuration

### Proprietary Components (Commercial License)
- Core semantic reasoning engine
- Advanced pattern recognition
- Intervention decision matrices
- Performance optimizations
- Binary distribution rights

### Compliance
- **Export Control**: Some algorithms may be restricted
- **Access Control**: Proprietary component authentication
- **Audit Trail**: All proprietary modifications logged
- **License Compliance**: Redistribution requires agreement

## Troubleshooting

### Development Issues
1. **Feature Conflicts**: Ensure only one feature enabled at build time
2. **Import Errors**: Check conditional compilation in main.rs
3. **Type Mismatches**: Verify common_types.rs consistency
4. **Linking Issues**: Check proprietary library availability

### Performance Issues
1. **High Latency**: Monitor logic execution times
2. **Memory Leaks**: Check for increasing memory usage
3. **Network Bottlenecks**: Verify UDP buffer sizes
4. **Intervention Failures**: Check system health logic
2. Verify Cyber-Monitor listens on 5556
3. Ensure Ghost-Node sends to 5555

## License

This project is part of the SAMS ecosystem and follows the same licensing terms.
