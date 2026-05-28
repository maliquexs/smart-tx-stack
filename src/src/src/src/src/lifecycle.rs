/// Transaction lifecycle tracking
/// 
/// This module will handle:
/// - End-to-end transaction tracking
/// - Stage transitions (Submitted → Processed → Confirmed → Finalized)
/// - Latency measurement
/// - Reorg detection
/// 
/// Phase 4 will implement the full tracking engine

use crate::types::{LifecycleEvent, LifecycleStage};
use anyhow::Result;
use std::collections::HashMap;

/// Lifecycle tracker: maintains state of all in-flight transactions
pub struct LifecycleTracker {
    // In Phase 4: HashMap of signature → LifecycleEvent chain
    events: HashMap<String, Vec<LifecycleEvent>>,
}

impl LifecycleTracker {
    /// Create a new lifecycle tracker
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    /// Record a lifecycle event
    pub fn record_event(&mut self, event: LifecycleEvent) -> Result<()> {
        self.events
            .entry(event.signature.clone())
            .or_insert_with(Vec::new)
            .push(event);
        Ok(())
    }

    /// Get lifecycle events for a transaction
    pub fn get_events(&self, signature: &str) -> Option<&Vec<LifecycleEvent>> {
        self.events.get(signature)
    }

    /// Get the latest stage for a transaction
    pub fn latest_stage(&self, signature: &str) -> Option<LifecycleStage> {
        self.events
            .get(signature)
            .and_then(|events| events.last())
            .map(|event| event.stage.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_tracker_creation() {
        let _tracker = LifecycleTracker::new();
        // Phase 4: actual tests
    }
}
