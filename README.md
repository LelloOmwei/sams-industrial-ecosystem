# SAMS Industrial Ecosystem

A comprehensive monorepo containing the core SAMS (Semantic Atom Monitoring System) applications for Industry 4.0 industrial automation and monitoring.

## 🏗️ Architecture Overview

This ecosystem consists of four specialized applications that work together to provide a complete industrial monitoring and control solution:

### 📦 Core Applications

#### 1. **Ghost-Node** (`sams-ghost-node/`)
- **Purpose**: Industrial sensor data generator with microjoule energy tracking
- **Technology**: Rust, TUI (Terminal User Interface)
- **Features**: Sub-microsecond latency monitoring, energy-efficient data generation

#### 2. **Semantic Logic Controller** (`sams-logic-gate/`)
- **Purpose**: Professional middleware with hybrid open-source/proprietary architecture
- **Technology**: Rust, Advanced semantic processing
- **Features**: Pattern recognition, intervention logic, performance optimization
- **Note**: Contains proprietary IP protection components

#### 3. **Cyber-Monitor** (`cyber-monitor/`)
- **Purpose**: Real-time TUI dashboard for system monitoring with semantic query engine
- **Technology**: Rust, Terminal-based monitoring, Zero-copy binary filtering
- **Features**: Live system metrics, resource monitoring, semantic search, sub-microsecond query execution

#### 4. **Black-Box Auditor** (`sams-blackbox/`)
- **Purpose**: High-speed immutable binary logger for compliance and forensic audit
- **Technology**: Rust, Secure logging
- **Features**: Immutable audit trails, forensic analysis, industrial compliance

## 📊 Real-Time Semantic Query Engine

The cyber-monitor features a high-performance semantic query engine that enables real-time filtering of binary 32-byte atoms using natural language commands. This system performs zero-copy binary filtering directly on memory-mapped data structures for maximum efficiency.

### Query Capabilities

Filter industrial monitoring data with natural language commands:

```bash
# Technical queries
energy > 5
node = 42
type = 1 and energy > 10

# Natural language queries
high power              # Interpreted as: energy > 80
sensor 42              # Interpreted as: node = 42
hungry machine          # Interpreted as: energy > 80 and node = (any)
low consumption         # Interpreted as: energy < 20

# Multilingual support (Slovak)
spotreba > 10         # Interpreted as: energy > 10
senzor 42              # Interpreted as: node = 42
vysoke spotreba        # Interpreted as: energy > 80
```

### Performance Features

- **Zero-copy binary filtering** on 8th-byte energy offset
- **Sub-microsecond query execution** on 1000+ stored atoms
- **Natural Language Processing** with synonym recognition
- **TUI-integrated command line** with instant visual feedback
- **Multilingual support** for English and Slovak operators
- **Smart threshold mapping** (high=80, low=20, efficient=20)

## �� Quick Start

### Prerequisites
- Rust 1.70+ (recommended: latest stable)
- Terminal environment for TUI applications

### Building All Projects
```bash
# Build all projects
for dir in */; do cd "$dir" && cargo build --release && cd ..; done
```

### Running Individual Projects
```bash
# Ghost-Node (Sensor Data Generator)
cd sams-ghost-node && cargo run --release

# Semantic Logic Controller
cd sams-logic-gate && cargo run --release --features open-source

# Cyber-Monitor (Dashboard with Natural Language Search)
cd cyber-monitor && cargo run --release
# Press '/' to enter search mode, then type: high power or sensor 42

# Black-Box Auditor
cd sams-blackbox && cargo run --release
```

## 📋 System Requirements

- **OS**: Linux, macOS, or Windows (WSL2)
- **Memory**: Minimum 4GB RAM, 8GB+ recommended
- **Storage**: 1GB free space for builds
- **Terminal**: UTF-8 compatible terminal for TUI applications

## 🔧 Development

### Project Structure
```
sams-industrial-ecosystem/
├── sams-ghost-node/          # Sensor data generator
├── sams-logic-gate/          # Semantic logic controller
├── cyber-monitor/            # System monitoring dashboard
├── sams-blackbox/            # Binary auditor
├── README.md                 # This file
├── LICENSE                   # MIT License
└── .gitignore               # Git ignore rules
```

### Testing
```bash
# Run tests for all projects
for dir in */; do cd "$dir" && cargo test && cd ..; done
```

### Code Quality
```bash
# Format code
for dir in */; do cd "$dir" && cargo fmt && cd ..; done

# Run clippy
for dir in */; do cd "$dir" && cargo clippy -- -D warnings && cd ..; done
```

## 📄 Licensing

This monorepo contains components with different licenses:

- **Open Source Components**: MIT License
- **Proprietary Components**: Commercial license required
- **See individual project directories for specific licensing information**

## 🤝 Contributing

Contributions are welcome for open-source components. Please refer to individual project documentation for contribution guidelines.

## 📞 Support

For project information and support:
- **Email**: StanoL76@protonmail.com

## 🏢 About SAMS

**SAMS** (Semantic Atom Monitoring System) is an industrial monitoring ecosystem focused on:

- **32-byte binary protocol**: Efficient data transmission for industrial sensors
- **Energy-efficient monitoring**: Microjoule-level energy tracking and optimization
- **Real-time semantic processing**: Advanced pattern recognition and intervention logic
- **Industrial compliance**: Immutable audit trails and forensic analysis

---

**⚠️ Important**: This ecosystem contains proprietary intellectual property. Ensure proper licensing before commercial deployment.
