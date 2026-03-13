# SAMS Semantic Logic Controller (SLC) Integration Instructions

## Overview
The SAMS Semantic Logic Controller (SLC) serves as professional middleware between Ghost-Node and Cyber-Monitor, implementing advanced semantic processing, stateful logic, and intelligent intervention capabilities.

## Architecture Flow
```
Ghost-Node (5555) → SLC (5555→5556) → Cyber-Monitor (5556)
```

## Proprietary SLC Logic Protection

### 🔒 Intellectual Property Architecture
The SAMS ecosystem follows a **Hybrid Open-Source/Proprietary Model** where the transport protocol and infrastructure are open-source, but the core semantic reasoning engine remains proprietary intellectual property.

### 📋 Component Classification

#### Open-Source Components
- **Transport Protocol**: UDP networking and data flow
- **UI Framework**: Terminal interface and visualization
- **Integration Layer**: Generic networking and I/O handling
- **Mock Implementation**: `mock_logic.rs` for community development
- **Documentation**: Integration guides and API specifications

#### Proprietary Components
- **Core Logic Engine**: `slc_core.rs` - Semantic reasoning algorithms
- **Pattern Recognition**: Advanced heuristics and decision matrices
- **Intervention Logic**: Intelligent system protection mechanisms
- **Performance Optimizations**: Proprietary processing pipelines
- **Decision Algorithms**: Confidential semantic analysis methods

### 🏗️ Bridge Pattern Architecture

The SLC implements a pluggable architecture that allows seamless switching between open-source and proprietary logic cores:

```rust
// Build-time configuration
#[cfg(feature = "closed-source")]
mod slc_core;

#[cfg(feature = "open-source")]
mod mock_logic;

// Unified interface
use LogicController trait;
```

### 📜 Licensing Model

#### Open-Source License (MIT)
- Transport protocol and networking layer
- UI framework and visualization components  
- Mock implementation for community development
- Integration documentation and examples
- Build system and configuration files

#### Proprietary License (Commercial)
- Core semantic reasoning engine
- Advanced pattern recognition algorithms
- Intervention decision matrices
- Performance-optimized processing pipelines
- Binary distribution rights

### 🔧 Development Modes

#### Open-Source Development
```bash
# Default mode - uses mock logic
cargo build
cargo run

# Explicit open-source mode
cargo build --features "open-source"
```

#### Closed-Source Deployment
```bash
# Production mode - uses proprietary logic
cargo build --features "closed-source"

# Link with pre-compiled proprietary binary
# Place proprietary library in proprietary_blobs/
```

### 🛡️ IP Protection Measures

#### Code-Level Protection
- **Prominent Warnings**: Clear IP notices in proprietary files
- **Compilation Guards**: Feature-based conditional compilation
- **Binary Separation**: Proprietary blobs excluded from version control
- **License Headers**: Comprehensive IP protection notices

#### Repository Structure
```
sams-logic-gate/
├── src/
│   ├── main.rs              # Open-source orchestration
│   ├── mock_logic.rs        # Open-source implementation
│   └── slc_core.rs          # PROPRIETARY - DO NOT DISTRIBUTE
├── proprietary_blobs/          # Excluded from git
│   ├── .gitignore            # Prevents accidental commits
│   └── [proprietary libs]    # Pre-compiled binaries
├── Cargo.toml                # Feature-based configuration
└── INTEGRATION_SLC.md        # This documentation
```

#### Distribution Guidelines
- **Public Repositories**: Must exclude `slc_core.rs` and `proprietary_blobs/`
- **Open-Source Contributions**: Use `mock_logic.rs` for development
- **Commercial Distribution**: Include proprietary binaries with license
- **Binary Redistribution**: Requires explicit licensing agreement

### 🤝 Community Contribution Guidelines

#### Allowed Contributions
- UI improvements and new visualizations
- Networking layer optimizations
- Documentation and examples
- Mock logic enhancements
- Integration with new data sources
- Performance monitoring tools

#### Restricted Areas
- Core semantic reasoning algorithms
- Pattern recognition heuristics
- Intervention decision logic
- Proprietary optimization techniques

#### Contribution Process
1. **Fork Repository**: Create open-source fork
2. **Develop with Mock**: Use `mock_logic.rs` for development
3. **Test Thoroughly**: Ensure compatibility with open-source mode
4. **Submit Pull Request**: Target open-source components only
5. **Licensing Review**: All contributions must be MIT compatible

### ⚖️ Legal and Compliance

#### Intellectual Property Rights
- **Algorithm Ownership**: Core reasoning algorithms remain proprietary
- **Patent Protection**: Decision matrices and heuristics protected
- **Trade Secrets**: Performance optimizations confidential
- **Binary Licensing**: Commercial distribution requires agreement

#### Compliance Requirements
- **Export Control**: Some algorithms may be subject to export regulations
- **Access Control**: Proprietary components require authentication
- **Audit Trail**: All proprietary modifications must be logged
- **License Compliance**: Redistribution must respect IP boundaries

---

## Required Updates

### 1. Cyber-Monitor Configuration Update
Update your Cyber-Monitor to listen on port **5556** instead of 5555:

**In `cyber-monitor/src/main.rs`:**
```rust
// Change this line:
let socket = UdpSocket::bind("0.0.0.0:5555").await?;
// To:
let socket = UdpSocket::bind("0.0.0.0:5556").await?;
```

### 2. System Startup Order
1. **Start Cyber-Monitor first** (listening on port 5556)
2. **Start SAMS SLC** (listens on 5555, forwards to 5556)
3. **Start Ghost-Node** (sends to port 5555)

## Usage

### Starting the SLC
```bash
cd sams-logic-gate
cargo run
```

### UI Controls
- Press 'q' to quit the application
- Professional industrial interface with real-time metrics
- Color-coded system health indicators

### Enhanced Data Processing Rules
The SLC applies advanced semantic logic to incoming atoms:

1. **Input Validation**: Anti-replay protection and timestamp validation
2. **High Load Detection**: If `energy_cost > 100μJ`, adds "HIGH_LOAD" tag
3. **Security Check**: If `trust_pqc == false`, generates critical security alert
4. **Intervention Logic**: 3+ consecutive high-energy atoms trigger automatic intervention

### Professional Metrics Display
- **Processed**: Total atoms successfully processed
- **Rejected**: Atoms failing validation checks
- **Rules**: Total rule activations
- **High Load**: Atoms exceeding energy threshold
- **Security**: PQC trust failures detected
- **Interventions**: Automatic system protections applied
- **Health**: Current system state (Optimal/Warning/Critical/Intervention)
- **Avg Logic**: Sub-microsecond processing performance

## Enhanced Features

### 🔒 Security & Validation
- **Anti-Replay Protection**: Rejects duplicate atoms within 1-second windows
- **Timestamp Validation**: Ensures atoms are within ±5 minute time range
- **Integrity Checks**: Validates atom structure and required fields

### 🧠 Stateful Processing
- **System Health Tracking**: Monitors last 10 atoms for trend analysis
- **Health States**: 
  - 🟢 **Optimal**: Normal operation
  - 🟡 **Warning**: Elevated activity detected
  - 🔴 **Critical**: Multiple issues present
  - 🟣 **Intervention**: Automatic protection activated

### ⚡ Intelligent Intervention
- **Trigger Condition**: 3+ consecutive atoms with `energy_cost > 120μJ`
- **Warning Code**: Injects 0xDEAD into payload first 2 bytes
- **Cooldown**: 5-second minimum between interventions
- **Payload Modification**: Preserves data while adding warnings

## Testing

### Send Test Data
```bash
# Normal atom
echo '{"id":"test-1","timestamp":1234567890,"energy_cost":50.5,"trust_pqc":true,"data":{},"tags":[],"payload":[0,0,0,0,0,0,0,0]}' | nc -u localhost 5555

# High load atom
echo '{"id":"test-2","timestamp":1234567890,"energy_cost":150.0,"trust_pqc":true,"data":{},"tags":[],"payload":[0,0,0,0,0,0,0,0]}' | nc -u localhost 5555

# Security alert atom
echo '{"id":"test-3","timestamp":1234567890,"energy_cost":75.0,"trust_pqc":false,"data":{},"tags":[],"payload":[0,0,0,0,0,0,0,0]}' | nc -u localhost 5555

# Intervention trigger (send 3x high energy in sequence)
for i in {1..3}; do
  echo "{\"id\":\"intervene-$i\",\"timestamp\":1234567890,\"energy_cost\":130.0,\"trust_pqc\":true,\"data\":{},\"tags\":[],\"payload\":[0,0,0,0,0,0,0,0]}" | nc -u localhost 5555
  sleep 0.1
done
```

## Expected Behavior

### Normal Processing
1. **Valid atoms** pass through validation and processing
2. **Tags are added** based on rule evaluations
3. **Processed atoms** are forwarded to Cyber-Monitor
4. **Metrics update** in real-time

### Intervention Scenario
1. **3 consecutive high-energy atoms** trigger intervention
2. **Payload modified** with 0xDEAD warning code
3. **Intervention tag** added to atom
4. **System health** changes to Intervention state
5. **Cooldown period** prevents immediate re-intervention

### Security Events
1. **PQC trust failures** generate security alerts
2. **Security tags** added to affected atoms
3. **System health** may degrade based on patterns
4. **Metrics track** all security events

## Performance Monitoring

### Key Metrics
- **Avg Logic Time**: Should remain sub-microsecond (< 1μs)
- **Processing Rate**: Monitor throughput capability
- **Rejection Rate**: Should remain low (< 5%)
- **Intervention Frequency**: Track system protection events

### Health State Transitions
- **Optimal → Warning**: Elevated activity or minor issues
- **Warning → Critical**: Multiple concerning patterns
- **Critical → Intervention**: Automatic protection needed
- **Intervention → Optimal**: System recovers after cooldown

## Troubleshooting

### Port Conflicts
Ensure ports are available:
- 5555: SLC input from Ghost-Node
- 5556: Cyber-Monitor input

### Data Flow Issues
1. Check SLC receives data (watch processed metrics)
2. Verify Cyber-Monitor listens on 5556
3. Ensure Ghost-Node sends to 5555
4. Monitor rejection rate for validation issues

### Performance Issues
- Monitor "Avg Logic" metric for processing delays
- Check system resources (CPU, memory)
- Verify network connectivity
- Review intervention frequency

### Validation Failures
- Check timestamp synchronization across systems
- Verify atom ID uniqueness
- Ensure required fields are present
- Monitor replay protection map size

## Architecture Benefits

### Professional Industrial Design
- **Modular Architecture**: Clean separation between UI, networking, and logic
- **Trait-Based Design**: LogicController interface for extensibility
- **Stateful Processing**: Intelligent pattern recognition
- **Real-Time Performance**: Sub-microsecond processing latency

### Enterprise Features
- **Input Validation**: Comprehensive security checks
- **State Management**: Thread-safe shared state
- **Performance Monitoring**: Real-time execution tracking
- **Intervention Logic**: Automatic system protection

### Scalability
- **Async Processing**: Non-blocking UDP handling
- **Memory Efficiency**: Circular buffers and cleanup
- **High Throughput**: Thousands of atoms per second
- **Graceful Degradation**: Continues operation during issues

## License & Distribution

⚠️ **IMPORTANT**: The SLC contains proprietary logic components in `src/slc_core.rs` that are not licensed under open source terms. Ensure compliance with distribution restrictions.

---

**SAMS Semantic Logic Controller v0.1.0** - Professional Industrial-Grade Middleware
