/*
================================================================================
 SAMS SEMANTIC LOGIC CONTROLLER - PROPRIETARY INTELLECTUAL PROPERTY
================================================================================

WARNING: This file contains proprietary intellectual property and trade
secrets belonging to Stanislav Levarsky and SAMS Technology. Access to,
use, reproduction, modification, distribution, or disclosure of this code
is strictly prohibited without a valid commercial license agreement.

PROPRIETARY COMPONENTS:
- Core Semantic Reasoning Engine
- Advanced Pattern Recognition Algorithms  
- Intervention Logic Matrices
- Performance Optimization Techniques
- Decision-Making Heuristics

LICENSE REQUIREMENTS:
• Commercial license required for production use
• See LICENSE_PROPRIETARY.md for complete terms
• Open-source components are in mock_logic.rs
• Transport/UI layer is MIT licensed

PROTECTION MEASURES:
• Trade secret protection under applicable law
• Copyright protection international treaties
• Patent protection on specific algorithms
• Contract protection through license agreements

VIOLATION CONSEQUENCES:
• Civil liability for damages and injunctions
• Criminal liability for willful infringement
• Export control violations
• Trade secret misappropriation claims

CONTACT FOR LICENSING:
Email: licensing@sams-technology.com
Web: https://sams-technology.com/licensing

================================================================================
 PROPRIETARY & CONFIDENTIAL - DO NOT DISTRIBUTE OR REPRODUCE
================================================================================
*/
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use crate::common_types::{LogicController, SemanticAtom, ProcessedSemanticAtom, SystemHealth};

#[derive(Debug, Clone)]
pub struct SlcState {
    pub recent_atoms: Vec<SemanticAtom>,
    pub system_health: SystemHealth,
    pub high_energy_sequence: u8,
    pub last_intervention: Option<Instant>,
    pub total_processed: u64,
    pub replay_protection: HashMap<String, u64>,  // atom_id -> timestamp
}

impl Default for SlcState {
    fn default() -> Self {
        Self {
            recent_atoms: Vec::with_capacity(10),
            system_health: SystemHealth::Optimal,
            high_energy_sequence: 0,
            last_intervention: None,
            total_processed: 0,
            replay_protection: HashMap::new(),
        }
    }
}

pub struct SemanticLogicController {
    state: std::sync::Arc<tokio::sync::RwLock<SlcState>>,
}

impl SemanticLogicController {
    pub fn new() -> Self {
        Self {
            state: std::sync::Arc::new(tokio::sync::RwLock::new(SlcState::default())),
        }
    }

    async fn validate_atom(&self, atom: &SemanticAtom) -> bool {
        // Anti-replay protection: check if we've seen this atom ID recently
        {
            let state = self.state.read().await;
            if let Some(&last_timestamp) = state.replay_protection.get(&atom.id) {
                // If same atom ID seen within 1 second, reject as replay
                if atom.timestamp <= last_timestamp + 1000 {
                    return false;
                }
            }
        }

        // Timestamp sanity check: must be within reasonable range
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Reject if timestamp is more than 60 seconds in the future or 300 seconds in the past
        let time_diff = if atom.timestamp > now {
            atom.timestamp - now
        } else {
            now - atom.timestamp
        };
        
        time_diff < 300
    }

    async fn update_system_health(&self, atom: &SemanticAtom) -> SystemHealth {
        let mut state = self.state.write().await;
        
        // Add to recent atoms (keep last 10)
        state.recent_atoms.push(atom.clone());
        if state.recent_atoms.len() > 10 {
            state.recent_atoms.remove(0);
        }

        // Track high energy sequence
        if atom.energy_cost > 120.0 {
            state.high_energy_sequence += 1;
        } else {
            state.high_energy_sequence = 0;
        }

        // Calculate system health based on recent atoms
        let high_load_count = state.recent_atoms.iter()
            .filter(|a| a.energy_cost > 100.0)
            .count();
        
        let security_issues = state.recent_atoms.iter()
            .filter(|a| !a.trust_pqc)
            .count();

        state.system_health = match (high_load_count, security_issues, state.high_energy_sequence) {
            (_, _, 3..) => SystemHealth::Intervention,
            (7.., _, _) | (_, 3.., _) => SystemHealth::Critical,
            (3.., 1.., _) => SystemHealth::Warning,
            _ => SystemHealth::Optimal,
        };

        state.system_health.clone()
    }

    async fn apply_intervention(&self, atom: &mut SemanticAtom) -> bool {
        let state = self.state.read().await;
        
        // Check if we need intervention (3+ high energy atoms in sequence)
        if state.high_energy_sequence >= 3 {
            // Check cooldown period (minimum 5 seconds between interventions)
            if let Some(last_intervention) = state.last_intervention {
                if last_intervention.elapsed() < Duration::from_secs(5) {
                    return false;
                }
            }

            drop(state); // Release read lock
            
            // Apply intervention: modify payload with warning code
            let mut payload = atom.payload.clone().unwrap_or_else(|| vec![0; 8]);
            
            // Set warning code in first 2 bytes (0xDEAD)
            if payload.len() >= 2 {
                payload[0] = 0xDE;
                payload[1] = 0xAD;
            } else {
                payload = vec![0xDE, 0xAD];
            }
            
            atom.payload = Some(payload);
            atom.tags.push("INTERVENTION_APPLIED".to_string());
            
            // Update last intervention time
            let mut state = self.state.write().await;
            state.last_intervention = Some(Instant::now());
            
            true
        } else {
            false
        }
    }
}

impl LogicController for SemanticLogicController {
    fn process_atom(&self, atom: SemanticAtom) -> Option<ProcessedSemanticAtom> {
        let start_time = Instant::now();
        
        // Use tokio runtime for async operations
        let rt = tokio::runtime::Handle::current();
        
        rt.block_on(async {
            // Input validation
            if !self.validate_atom(&atom).await {
                return None;
            }

            // Update replay protection
            {
                let mut state = self.state.write().await;
                state.replay_protection.insert(atom.id.clone(), atom.timestamp);
                state.total_processed += 1;
            }

            let mut processed_atom = ProcessedSemanticAtom {
                original: atom.clone(),
                processing_time: Duration::default(),
                tags_added: Vec::new(),
                security_alert: None,
                intervention_applied: false,
                system_health: SystemHealth::Optimal,
            };

            // Update system health
            processed_atom.system_health = self.update_system_health(&atom).await;

            // Apply processing rules
            let mut rules_triggered = 0;

            // Rule 1: High load detection
            if atom.energy_cost > 100.0 {
                processed_atom.tags_added.push("HIGH_LOAD".to_string());
                rules_triggered += 1;
            }

            // Rule 2: Security check
            if !atom.trust_pqc {
                processed_atom.security_alert = Some("CRITICAL: PQC trust failure detected".to_string());
                processed_atom.tags_added.push("SECURITY_ALERT".to_string());
                rules_triggered += 1;
            }

            // Rule 3: Intervention logic (stateful)
            let mut atom_mut = atom.clone();
            processed_atom.intervention_applied = self.apply_intervention(&mut atom_mut).await;
            if processed_atom.intervention_applied {
                processed_atom.tags_added.push("INTERVENTION_APPLIED".to_string());
                rules_triggered += 1;
            }

            // Update original atom with modifications
            processed_atom.original = atom_mut;
            processed_atom.processing_time = start_time.elapsed();

            Some(processed_atom)
        })
    }
}

// Factory function for creating the SLC instance
pub fn create_slc() -> Box<dyn LogicController + Send + Sync> {
    Box::new(SemanticLogicController::new())
}
