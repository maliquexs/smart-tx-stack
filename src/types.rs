/// Core type definitions for Smart Transaction Stack
/// 
/// All types here are serializable (serde) for:
/// - JSON logging (AI agent reasoning visibility)
/// - State persistence
/// - Cross-module communication

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Current slot information from Yellowstone gRPC
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlotInfo {
    /// Absolute slot number
    pub slot: u64,
    /// Current Unix timestamp (seconds)
    pub timestamp: u64,
    /// Leader pubkey for this slot (base58)
    pub leader: String,
    /// Parent slot (for reorg detection)
    pub parent_slot: u64,
}

impl SlotInfo {
    /// Create a new SlotInfo with current timestamp
    pub fn new(slot: u64, leader: String, parent_slot: u64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            slot,
            timestamp,
            leader,
            parent_slot,
        }
    }
}

/// Leader schedule state (current + next leaders)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaderSchedule {
    /// Current leader for this slot
    pub current_leader: String,
    /// Upcoming leaders in order [next, next+1, next+2, ...]
    pub next_leaders: Vec<String>,
    /// Slot at which this schedule was observed
    pub observed_at_slot: u64,
}

impl LeaderSchedule {
    /// Create new leader schedule
    pub fn new(current_leader: String, next_leaders: Vec<String>, observed_at_slot: u64) -> Self {
        Self {
            current_leader,
            next_leaders,
            observed_at_slot,
        }
    }

    /// Get the leader at a relative offset from current
    /// offset=0 -> current_leader, offset=1 -> next_leaders[0], etc.
    pub fn leader_at_offset(&self, offset: usize) -> Option<String> {
        if offset == 0 {
            Some(self.current_leader.clone())
        } else {
            self.next_leaders.get(offset - 1).cloned()
        }
    }
}

/// Transaction lifecycle stage (for end-to-end tracking)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LifecycleStage {
    /// Submitted to Jito (or local mempool)
    Submitted,
    /// Accepted by bundle engine / validator
    Accepted,
    /// Shred propagated to network
    ShredPropagated,
    /// Processed by leader (in a block)
    Processed,
    /// Confirmed (1+ confirmations)
    Confirmed,
    /// Finalized (32+ confirmations)
    Finalized,
}

impl std::fmt::Display for LifecycleStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Submitted => write!(f, "SUBMITTED"),
            Self::Accepted => write!(f, "ACCEPTED"),
            Self::ShredPropagated => write!(f, "SHRED_PROPAGATED"),
            Self::Processed => write!(f, "PROCESSED"),
            Self::Confirmed => write!(f, "CONFIRMED"),
            Self::Finalized => write!(f, "FINALIZED"),
        }
    }
}

/// Lifecycle event: stage transition with timestamp + latency tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifecycleEvent {
    /// Transaction signature (base58)
    pub signature: String,
    /// Stage this event represents
    pub stage: LifecycleStage,
    /// Absolute Unix timestamp (ms) when this stage was reached
    pub timestamp_ms: u64,
    /// Latency from Submitted to this stage (ms). None if stage < Submitted
    pub latency_from_submit_ms: Option<u64>,
    /// Slot where this event occurred
    pub slot: u64,
    /// Optional metadata (error details, commitment level, etc.)
    pub metadata: Option<String>,
}

impl LifecycleEvent {
    /// Create a new lifecycle event
    pub fn new(
        signature: String,
        stage: LifecycleStage,
        slot: u64,
        latency_from_submit_ms: Option<u64>,
    ) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            signature,
            stage,
            timestamp_ms,
            latency_from_submit_ms,
            slot,
            metadata: None,
        }
    }

    /// Attach metadata to this event
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Failure classification (deterministic taxonomy for AI agent)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FailureType {
    /// Tip too low relative to network demand
    FeeTooLow {
        submitted_tip: u64,
        suggested_tip: u64,
    },
    /// Blockhash expired (older than 150 blocks)
    BlockhashExpired {
        submitted_blockhash_age: u64,
    },
    /// Compute budget exceeded (simulated failure)
    ComputeExceeded {
        compute_used: u64,
        compute_limit: u64,
    },
    /// Bundle dropped by validator (no shred propagation)
    BundleDropped,
    /// Leader skipped (leader down or slow)
    LeaderSkip {
        expected_slot: u64,
        actual_parent: u64,
    },
    /// Transaction reorg'd (confirmed then dropped)
    ReorgVictim {
        originally_at_slot: u64,
        reorg_depth: u64,
    },
    /// Jito rejected (API error or capacity)
    JitoRejection {
        reason: String,
    },
    /// Network timeout (no response within SLA)
    NetworkTimeout {
        elapsed_ms: u64,
    },
    /// Other / unclassified
    Unknown {
        reason: String,
    },
}

impl std::fmt::Display for FailureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FeeTooLow { .. } => write!(f, "FEE_TOO_LOW"),
            Self::BlockhashExpired { .. } => write!(f, "BLOCKHASH_EXPIRED"),
            Self::ComputeExceeded { .. } => write!(f, "COMPUTE_EXCEEDED"),
            Self::BundleDropped => write!(f, "BUNDLE_DROPPED"),
            Self::LeaderSkip { .. } => write!(f, "LEADER_SKIP"),
            Self::ReorgVictim { .. } => write!(f, "REORG_VICTIM"),
            Self::JitoRejection { .. } => write!(f, "JITO_REJECTION"),
            Self::NetworkTimeout { .. } => write!(f, "NETWORK_TIMEOUT"),
            Self::Unknown { .. } => write!(f, "UNKNOWN"),
        }
    }
}

/// Bundle submission record (for later phases)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleSubmission {
    /// Unique bundle ID (UUID or signature-based)
    pub bundle_id: String,
    /// Transaction signatures in this bundle
    pub signatures: Vec<String>,
    /// Tip paid (lamports)
    pub tip_lamports: u64,
    /// Target slot for landing
    pub target_slot: u64,
    /// Slot when submitted
    pub submitted_at_slot: u64,
    /// Submission timestamp (ms)
    pub submitted_at_ms: u64,
    /// Current stage
    pub stage: LifecycleStage,
    /// Retry count
    pub retry_count: u32,
}

impl BundleSubmission {
    /// Create a new bundle submission
    pub fn new(
        bundle_id: String,
        signatures: Vec<String>,
        tip_lamports: u64,
        target_slot: u64,
        submitted_at_slot: u64,
    ) -> Self {
        let submitted_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            bundle_id,
            signatures,
            tip_lamports,
            target_slot,
            submitted_at_slot,
            submitted_at_ms,
            stage: LifecycleStage::Submitted,
            retry_count: 0,
        }
    }
}

/// Metrics snapshot (for observability)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Slot when snapshot was taken
    pub slot: u64,
    /// Total bundles submitted in this epoch (last ~5 min)
    pub bundles_submitted: u32,
    /// Bundles confirmed
    pub bundles_confirmed: u32,
    /// Bundles failed
    pub bundles_failed: u32,
    /// Average tip paid (lamports)
    pub avg_tip_lamports: u64,
    /// P95 latency (submitted -> confirmed) in ms
    pub latency_p95_ms: u64,
    /// Current network demand estimate (1.0 = baseline)
    pub demand_multiplier: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_info_creation() {
        let info = SlotInfo::new(12345, "Leader1".to_string(), 12344);
        assert_eq!(info.slot, 12345);
        assert_eq!(info.leader, "Leader1");
        assert_eq!(info.parent_slot, 12344);
        assert!(info.timestamp > 0);
    }

    #[test]
    fn test_leader_schedule_offset() {
        let schedule = LeaderSchedule::new(
            "Leader0".to_string(),
            vec!["Leader1".to_string(), "Leader2".to_string()],
            100,
        );
        assert_eq!(schedule.leader_at_offset(0), Some("Leader0".to_string()));
        assert_eq!(schedule.leader_at_offset(1), Some("Leader1".to_string()));
        assert_eq!(schedule.leader_at_offset(2), Some("Leader2".to_string()));
        assert_eq!(schedule.leader_at_offset(3), None);
    }

    #[test]
    fn test_failure_type_display() {
        let failure = FailureType::FeeTooLow {
            submitted_tip: 1000,
            suggested_tip: 2000,
        };
        assert_eq!(failure.to_string(), "FEE_TOO_LOW");
    }

    #[test]
    fn test_lifecycle_event_serialization() {
        let event = LifecycleEvent::new(
            "sig123".to_string(),
            LifecycleStage::Accepted,
            100,
            Some(50),
        );
        let json = serde_json::to_string(&event).expect("serialization failed");
        assert!(json.contains("sig123"));
        assert!(json.contains("ACCEPTED"));
    }
      }
          
