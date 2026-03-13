# SAMS Semantic Logic Controller (SLC) v0.1.0

**Professional Industrial-Grade Logic Controller** - A real-time semantic processing middleware for SAMS Atoms

## Overview

The SAMS Semantic Logic Controller (SLC) is a professional-grade middleware processor that implements advanced semantic logic, stateful processing, and intelligent intervention capabilities for SAMS Atoms flowing between Ghost-Node and Cyber-Monitor.

## Architecture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ GHOST-NODE  │───▶│ SLC CORE    │───▶│ DASHBOARD   │
│   (5555)    │    │  (PROCESS)  │    │   (5556)    │
└─────────────┘    └─────────────┘    └─────────────┘
       │                  │                  │
       ▼                  ▼                  ▼
   SAMS Atoms    Semantic Logic   Processed Data
                    ┌─────────────┐
                    │ VALIDATION  │
                    │ STATEFUL    │
                    │ INTERVENTION│
                    └─────────────┘
```

## Professional Features

### 🔒 **Input Validation & Security**
- **Anti-Replay Protection**: Prevents duplicate atom processing within 1-second windows
- **Timestamp Validation**: Rejects atoms with timestamps outside ±5 minute range
- **Integrity Checks**: Validates atom structure and required fields

### 🧠 **Stateful Logic Processing**
- **System Health Monitoring**: Tracks system state based on last 10 atoms
- **Health States**: Optimal → Warning → Critical → Intervention
- **Trend Analysis**: Detects patterns and anomalies in data streams

### ⚡ **Intelligent Intervention**
- **Automatic Intervention**: Triggers when 3+ consecutive atoms exceed 120μJ
- **Warning Code Injection**: Modifies payload with 0xDEAD header
- **Cooldown Period**: 5-second minimum between interventions
- **Payload Modification**: Preserves data integrity while adding warnings

### 🎯 **Advanced Processing Rules**
1. **High Load Detection**: `energy_cost > 100μJ` → "HIGH_LOAD" tag
2. **Security Analysis**: `trust_pqc == false` → critical security alert
3. **Intervention Logic**: Pattern-based automatic system protection

## Performance Metrics

### Real-Time Monitoring
- **Logic Execution Time**: Sub-microsecond processing latency
- **Throughput**: Atoms processed per second
- **Rejection Rate**: Invalid/filtered atoms
- **Intervention Count**: Automatic system protections
- **System Health**: Current operational state

### Industrial UI Theme
- **Deep Cyan/Slate**: Professional controller aesthetic
- **Color-Coded Health**: Green (Optimal), Yellow (Warning), Red (Critical), Magenta (Intervention)
- **Real-Time Updates**: 50ms refresh rate for live monitoring

## Quick Start

### Prerequisites
- Rust 1.70+
- Tokio async runtime

### Installation & Running
```bash
# Build the SLC
cd sams-logic-gate
cargo build --release

# Start the controller
cargo run
```

### Controls
- Press `q` to quit the application
- Real-time metrics display
- Professional industrial interface

## Data Format

### Input Semantic Atom
```json
{
  "id": "semantic-atom-001",
  "timestamp": 1234567890,
  "energy_cost": 75.5,
  "trust_pqc": true,
  "data": {},
  "tags": [],
  "payload": [0, 0, 0, 0, 0, 0, 0, 0]
}
```

### Processed Output
```json
{
  "id": "semantic-atom-001",
  "timestamp": 1234567890,
  "energy_cost": 75.5,
  "trust_pqc": true,
  "data": {},
  "tags": ["HIGH_LOAD", "SECURITY_ALERT"],
  "payload": [222, 173, 0, 0, 0, 0, 0, 0]  // 0xDEAD warning code
}
```

## System Integration

### Startup Sequence
1. **Cyber-Monitor** (listening on port 5556)
2. **SAMS SLC** (listens on 5555, forwards to 5556)
3. **Ghost-Node** (sends to port 5555)

### Cyber-Monitor Configuration
```rust
// Update port from 5555 to 5556
let socket = UdpSocket::bind("0.0.0.0:5556").await?;
```

## Testing & Validation

### Test Data Scenarios
```bash
# Normal atom
echo '{"id":"test-1","timestamp":1234567890,"energy_cost":50.5,"trust_pqc":true,"data":{},"tags":[],"payload":[0,0,0,0,0,0,0,0]}' | nc -u localhost 5555

# High load atom
echo '{"id":"test-2","timestamp":1234567890,"energy_cost":150.0,"trust_pqc":true,"data":{},"tags":[],"payload":[0,0,0,0,0,0,0,0]}' | nc -u localhost 5555

# Security alert atom
echo '{"id":"test-3","timestamp":1234567890,"energy_cost":75.0,"trust_pqc":false,"data":{},"tags":[],"payload":[0,0,0,0,0,0,0,0]}' | nc -u localhost 5555

# Intervention trigger (send 3x high energy)
for i in {1..3}; do
  echo "{\"id\":\"test-$i\",\"timestamp\":1234567890,\"energy_cost\":130.0,\"trust_pqc\":true,\"data\":{},\"tags\":[],\"payload\":[0,0,0,0,0,0,0,0]}" | nc -u localhost 5555
  sleep 0.1
done
```

## Architecture Details

### Core Components
- **slc_core.rs**: Proprietary logic controller implementation
- **LogicController Trait**: Generic interface for logic processing
- **SemanticLogicController**: Main SLC implementation
- **State Management**: Thread-safe shared state with Arc<RwLock>

### Performance Characteristics
- **Sub-microsecond Logic**: Average processing time < 1μs
- **High Throughput**: Thousands of atoms per second
- **Low Memory Footprint**: Efficient circular buffers
- **Async Processing**: Non-blocking UDP handling

### Memory Management
- **Recent Atoms Buffer**: Last 10 atoms for trend analysis
- **Logic Execution History**: Last 100 execution times for performance monitoring
- **Replay Protection Map**: Atom ID to timestamp mapping
- **State Tracking**: System health and intervention state

## Troubleshooting

### Common Issues
1. **Port Conflicts**: Ensure 5555/5556 are available
2. **High Rejection Rate**: Check timestamp synchronization
3. **Performance Degradation**: Monitor logic execution times
4. **Intervention Not Triggering**: Verify energy threshold sequence

### Performance Monitoring
- Watch "Avg Logic" metric - should stay < 1μs
- Monitor "Health" state transitions
- Track "Interventions" for system protection events
- Check "Rejected" atoms for validation issues

## Security & Compliance

### Protection Features
- **Replay Attack Prevention**: Duplicate atom detection
- **Timestamp Validation**: Temporal integrity checks
- **Payload Integrity**: Warning code injection for anomalies
- **Stateful Analysis**: Pattern-based threat detection

### Industrial Reliability
- **Graceful Degradation**: Continues processing during partial failures
- **State Recovery**: Maintains system health across restarts
- **Performance Monitoring**: Real-time execution tracking
- **Professional UI**: Industrial control interface standards

## License & Distribution

⚠️ **IMPORTANT**: This software contains proprietary SLC logic components that are not licensed under open source terms. See `src/slc_core.rs` for specific licensing restrictions.

---

**SAMS Semantic Logic Controller v0.1.0** - Professional Industrial-Grade Processing Middleware
