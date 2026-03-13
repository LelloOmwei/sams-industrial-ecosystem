# SAMS Cyber-Monitor Integration Plan

## Overview

This document outlines the three-phase transition strategy for evolving the cyber-monitor dashboard from a simulated demonstration to a real-time SAMS (Semantic Atom Management System) monitoring tool. The current implementation serves as a high-fidelity prototype that validates the UI/UX design and establishes the visual framework for future production deployment.

## Phase 1: From Random to Real (Milestone T1-T2)

### Current State
The dashboard currently uses `rand::random()` calls to generate simulated system metrics, security events, and energy calculations. This approach provides realistic visual feedback but operates independently of actual SAMS protocol data.

### Transition Architecture

#### Data Source Replacement
- **Objective**: Replace all `rand::random()` calls with real-time SAMS data ingestion
- **Implementation**: Implement either of the following data acquisition methods:

**Option A: Shared Memory (SHM) Interface**
```rust
// Future SHM reader structure
struct SamsShmReader {
    shm_segment: SharedMemory,
    atom_buffer: Vec<SemanticAtom>,
    read_offset: usize,
}
```

**Option B: Unix Domain Socket Listener**
```rust
// Future UDS listener structure
struct SamsUdsListener {
    socket: UnixListener,
    atom_stream: BufReader<UnixStream>,
}
```

#### Expected Input Format
The dashboard will consume a continuous stream of `SemanticAtom` structures:

```rust
#[repr(C)]
struct SemanticAtom {
    timestamp: u64,           // Unix epoch nanoseconds
    atom_id: u32,            // Unique atom identifier
    payload: [u8; 24],       // 24-byte semantic payload
    trust_pqc: bool,          // PQC verification flag
    energy_cost: u32,          // Measured energy cost (nanojoules)
    source_node: u16,         // Origin node identifier
}
```

#### Integration Points
1. **CPU Metrics**: Replace simulated CPU usage with actual system monitoring via `sysinfo` crate
2. **Memory Metrics**: Maintain current `sysinfo`-based memory monitoring
3. **Log Events**: Replace random log generation with parsed `SemanticAtom` events
4. **Energy Calculations**: Use `energy_cost` field from incoming atoms instead of static constants

## Phase 2: Hardware-Linked Energy Profiling

### Current Limitations
The dashboard uses static energy constants:
- SAMS energy cost: 12μJ/atom
- JSON energy cost: 180μJ/atom

### Dynamic Profiling Implementation

#### Hardware Counter Integration
- **x86 Systems**: Utilize Time Stamp Counter (TSC) for cycle-accurate measurements
- **ARM Systems**: Leverage Performance Monitoring Unit (PMU) for energy consumption tracking
- **Cross-Platform**: Abstract through `hw_counter` crate or custom implementation

#### Profiling Architecture
```rust
// Future energy profiler structure
struct SamsEnergyProfiler {
    baseline_cycles: u64,
    sams_codec_cycles: u64,
    json_codec_cycles: u64,
    energy_per_cycle: f64,  // Platform-specific calibration
}

impl SamsEnergyProfiler {
    fn profile_sams_operation(&mut self, atom: &SemanticAtom) -> u64 {
        let start = self.read_cycles();
        // Perform actual SamsCodec operation
        let end = self.read_cycles();
        let cycles = end - start;
        self.sams_codec_cycles += cycles;
        (cycles as f64 * self.energy_per_cycle) as u64
    }
}
```

#### Calibration Process
1. **Baseline Measurement**: Establish idle system energy consumption
2. **Operation Profiling**: Measure actual energy cost of SamsCodec operations
3. **Comparison Analysis**: Profile equivalent JSON encoding/decoding operations
4. **Dynamic Updates**: Continuously update energy efficiency metrics based on real measurements

#### Metrics Enhancement
- Replace static microjoule calculations with real-time hardware measurements
- Implement adaptive efficiency tracking based on actual workload patterns
- Add energy trend analysis and prediction capabilities

## Phase 3: Security & Trust Visualization

### Current Implementation
- **Encryption Toggle** (`e` key): Simple boolean flag manipulation
- **Security Scan** (`s` key): Simulated log message generation
- **System Encrypted Status**: Blinking indicator with no actual verification

### Trust-Linked Security Integration

#### PQC Anchor Verification
The dashboard will implement actual verification of Post-Quantum Cryptography anchors embedded in incoming `SemanticAtom` structures:

```rust
// Future PQC verification module
struct PqcVerifier {
    trusted_anchors: Vec<[u8; 32]>,
    verification_cache: LruCache<u32, bool>,
}

impl PqcVerifier {
    fn verify_atom(&mut self, atom: &SemanticAtom) -> bool {
        // Check trust_pqc flag against actual PQC signature verification
        if atom.trust_pqc {
            self.verify_signature(atom)
        } else {
            false
        }
    }
}
```

#### Security State Management
- **Real-time Verification**: Continuously verify PQC signatures of incoming atoms
- **Trust Level Visualization**: Color-coded indicators based on verification success rate
- **Anomaly Detection**: Alert on patterns of failed verifications or suspicious activity

#### Enhanced Security Features
1. **Trust Score Dashboard**: Real-time PQC verification success percentage
2. **Anchor Status Display**: Active/inactive PQC anchor information
3. **Security Event Correlation**: Link security events to specific atoms or nodes
4. **Threat Intelligence**: Integration with external security feeds for advanced threat detection

## Technical Architecture Guidelines

### UI/Data Decoupling
The ratatui UI logic is intentionally decoupled from data sources to facilitate seamless switching between simulation and live modes:

```rust
// Trait-based data source abstraction
trait SamsDataSource {
    fn get_atoms(&self) -> Vec<SemanticAtom>;
    fn get_system_metrics(&self) -> SystemMetrics;
    fn get_energy_metrics(&self) -> EnergyMetrics;
}

// Implementation for simulation mode
struct SimulatedDataSource;
impl SamsDataSource for SimulatedDataSource { /* ... */ }

// Implementation for live SAMS mode
struct LiveSamsDataSource;
impl SamsDataSource for LiveSamsDataSource { /* ... */ }
```

### Configuration Management
- **Mode Selection**: Runtime switching between "Simulated Mode" and "Live SAMS Mode"
- **Data Source Configuration**: Flexible configuration for SHM, UDS, or network sources
- **Performance Tuning**: Adjustable refresh rates and buffer sizes based on system capabilities

### Error Handling & Resilience
- **Graceful Degradation**: Automatic fallback to simulated mode on data source failure
- **Connection Recovery**: Automatic reconnection to SAMS data sources
- **Data Validation**: Input validation and sanitization for incoming atom streams

## Development Roadmap

### Milestone T1 (Weeks 1-2)
- Implement SHM/UDS data ingestion framework
- Define `SemanticAtom` data structures
- Replace random log generation with atom-based events
- Basic integration testing with simulated data streams

### Milestone T2 (Weeks 3-4)
- Integrate hardware performance counters
- Implement dynamic energy profiling
- Replace static energy constants with real measurements
- Add calibration routines for different hardware platforms

### Milestone T3 (Weeks 5-6)
- Implement PQC verification algorithms
- Link security visualizations to actual trust metrics
- Add advanced security event correlation
- Complete integration testing with live SAMS infrastructure

## Future Considerations

### Scalability
- Support for multiple concurrent SAMS streams
- Distributed deployment capabilities
- Load balancing for high-throughput environments

### Extensibility
- Plugin architecture for custom data sources
- Configurable visualization components
- API for external monitoring tool integration

### Performance Optimization
- GPU-accelerated rendering for large datasets
- Memory-mapped file I/O for high-throughput scenarios
- Adaptive refresh rates based on system load

---

*This integration plan serves as a technical roadmap for transforming the cyber-monitor dashboard from a demonstration prototype into a production-ready SAMS monitoring and analysis tool.*
