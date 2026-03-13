# SAMS Logic-Gate Integration Instructions

## Overview
The SAMS Logic-Gate acts as a middleware processor between Ghost-Node and Cyber-Monitor, implementing real-time data transformation and rule-based processing.

## Architecture Flow
```
Ghost-Node (5555) → Logic-Gate (5555→5556) → Cyber-Monitor (5556)
```

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
2. **Start SAMS Logic-Gate** (listens on 5555, forwards to 5556)
3. **Start Ghost-Node** (sends to port 5555)

## Usage

### Starting Logic-Gate
```bash
cd sams-logic-gate
cargo run
```

### UI Controls
- Press 'q' to quit the application
- The UI displays real-time metrics and processing status

### Data Processing Rules
The Logic-Gate applies these rules to incoming SAMS Atoms:

1. **High Load Detection**: If `energy_cost > 100μJ`, adds "HIGH_LOAD" tag
2. **Security Check**: If `trust_pqc == false`, generates critical security alert and adds "SECURITY_ALERT" tag

### Metrics Display
- **Atoms Processed**: Total number of atoms received and processed
- **Rules Triggered**: Count of all rule activations
- **High Load**: Number of atoms with energy cost > 100μJ
- **Security Alerts**: Number of atoms with PQC trust failures
- **Avg Latency**: Average processing time in microseconds

## Testing

### Send Test Data
You can test the system by sending JSON data to port 5555:

```bash
# Normal atom
echo '{"id":"test-1","timestamp":1234567890,"energy_cost":50.5,"trust_pqc":true,"data":{},"tags":[]}' | nc -u localhost 5555

# High load atom
echo '{"id":"test-2","timestamp":1234567890,"energy_cost":150.0,"trust_pqc":true,"data":{},"tags":[]}' | nc -u localhost 5555

# Security alert atom
echo '{"id":"test-3","timestamp":1234567890,"energy_cost":75.0,"trust_pqc":false,"data":{},"tags":[]}' | nc -u localhost 5555
```

## Expected Behavior
1. **Normal atoms** pass through unchanged
2. **High load atoms** get tagged with "HIGH_LOAD"
3. **Security atoms** get tagged with "SECURITY_ALERT" and trigger alerts
4. All processed atoms are forwarded to Cyber-Monitor on port 5556

## Troubleshooting

### Port Conflicts
Ensure ports are available:
- 5555: Logic-Gate input
- 5556: Cyber-Monitor input (was 5555)

### Data Flow Issues
1. Check Logic-Gate is receiving data (watch metrics)
2. Verify Cyber-Monitor is listening on 5556
3. Ensure Ghost-Node is sending to 5555

### Performance Monitoring
Watch the "Avg Latency" metric - should stay low (microseconds) for optimal performance.
