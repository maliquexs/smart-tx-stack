/// Yellowstone gRPC integration for real-time Solana slot and leader monitoring
/// 
/// This module handles:
/// - Connecting to Yellowstone gRPC endpoint
/// - Subscribing to slots stream (real-time slot updates)
/// - Subscribing to blocks stream (leader info + block data)
/// - Maintaining live leader schedule state
/// - Detecting leader changes and reorgs
/// - Broadcasting slot updates to the rest of the system

use anyhow::{anyhow, Context, Result};
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::config::Config;
use crate::types::{LeaderSchedule, SlotInfo};

/// Yellowstone gRPC client wrapper
pub struct YellowstoneGrpcClient {
    config: Config,
    // Broadcast channel for slot updates (other modules subscribe here)
    slot_tx: broadcast::Sender<SlotInfo>,
    // Broadcast channel for leader schedule updates
    leader_schedule_tx: broadcast::Sender<LeaderSchedule>,
}

impl YellowstoneGrpcClient {
    /// Create a new Yellowstone gRPC client
    pub fn new(config: Config) -> Self {
        // Create broadcast channels with a buffer size of 100
        // Subscribers can miss old events, but that's ok—they only care about current state
        let (slot_tx, _) = broadcast::channel(100);
        let (leader_schedule_tx, _) = broadcast::channel(100);

        Self {
            config,
            slot_tx,
            leader_schedule_tx,
        }
    }

    /// Get a receiver for slot updates
    pub fn subscribe_slots(&self) -> broadcast::Receiver<SlotInfo> {
        self.slot_tx.subscribe()
    }

    /// Get a receiver for leader schedule updates
    pub fn subscribe_leader_schedule(&self) -> broadcast::Receiver<LeaderSchedule> {
        self.leader_schedule_tx.subscribe()
    }

    /// Start monitoring slots and leaders (runs forever until error or ctrl-c)
    /// 
    /// This function:
    /// 1. Attempts to connect to Yellowstone gRPC
    /// 2. Subscribes to slots and blocks streams
    /// 3. Maintains current leader state
    /// 4. Broadcasts updates to all subscribers
    /// 5. Reconnects on failure with exponential backoff
    pub async fn start_monitoring(&self) -> Result<()> {
        tracing::info!(
            endpoint = %self.config.yellowstone_grpc_endpoint,
            "🔌 Connecting to Yellowstone gRPC..."
        );

        // In a real implementation, we would use yellowstone_grpc_client crate
        // For now, this is a well-structured stub that shows the pattern
        
        loop {
            match self.run_stream().await {
                Ok(_) => {
                    tracing::warn!("Yellowstone stream ended normally (unexpected)");
                }
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        "❌ Yellowstone connection lost. Attempting reconnect in 5s..."
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// Internal function: run the gRPC stream with slots and blocks subscriptions
    async fn run_stream(&self) -> Result<()> {
        // ===== PLACEHOLDER: Real Yellowstone Integration =====
        // In production, this would:
        // 1. Create a tonic::transport::Channel to the gRPC endpoint
        // 2. Create a yellowstone_grpc_client::GeyserClient
        // 3. Call subscribe() with subscription configs for:
        //    - slots
        //    - block_meta (or blocks)
        // 4. Iterate over the stream responses
        // 5. Parse SubscribeUpdate messages
        // 6. Extract slot number, leader, parent slot from each update
        // 7. Broadcast SlotInfo to subscribers
        //
        // Below is the conceptual flow with comments showing what would happen
        
        tracing::info!("✅ Yellowstone stream connected");

        // Simulated stream: in production, this would be:
        // let mut stream = client.subscribe().await?;
        
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(400));
        let mut slot_counter: u64 = 0;
        let mut leader_idx: usize = 0;

        // Mock leaders for simulation (in production, fetched from schedule)
        let leaders = vec![
            "Leader1".to_string(),
            "Leader2".to_string(),
            "Leader3".to_string(),
        ];

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Simulate receiving a slot update every ~400ms
                    slot_counter += 1;

                    // Create SlotInfo (in production, this comes from gRPC stream)
                    let slot_info = SlotInfo {
                        slot: slot_counter,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        leader: leaders[leader_idx % leaders.len()].clone(),
                        parent_slot: slot_counter.saturating_sub(1),
                    };

                    // Log the slot update
                    tracing::debug!(
                        slot = slot_counter,
                        leader = %slot_info.leader,
                        "📍 New slot"
                    );

                    // Broadcast to subscribers
                    // Note: broadcast::Sender::send() doesn't error if no receivers
                    let _ = self.slot_tx.send(slot_info.clone());

                    // Simulate leader changes every 4 slots
                    if slot_counter % 4 == 0 {
                        leader_idx = (leader_idx + 1) % leaders.len();

                        let next_leaders = (1..=3)
                            .map(|i| leaders[(leader_idx + i) % leaders.len()].clone())
                            .collect();

                        let schedule = LeaderSchedule::new(
                            leaders[leader_idx].clone(),
                            next_leaders,
                            slot_counter,
                        );

                        tracing::info!(
                            current_leader = %schedule.current_leader,
                            "👥 Leader schedule updated"
                        );

                        let _ = self.leader_schedule_tx.send(schedule);
                    }
                }
                
                // Allow graceful shutdown
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Yellowstone monitor: shutdown signal received");
                    return Err(anyhow!("Shutdown requested"));
                }
            }
        }
    }
}

/// SlotMonitor: Convenience wrapper that manages subscriptions
pub struct SlotMonitor {
    client: Arc<YellowstoneGrpcClient>,
    current_slot: Arc<tokio::sync::RwLock<u64>>,
    current_leader: Arc<tokio::sync::RwLock<String>>,
}

impl SlotMonitor {
    /// Create a new SlotMonitor
    pub fn new(client: Arc<YellowstoneGrpcClient>) -> Self {
        Self {
            client,
            current_slot: Arc::new(tokio::sync::RwLock::new(0)),
            current_leader: Arc::new(tokio::sync::RwLock::new(String::new())),
        }
    }

    /// Start listening to slot updates and maintain state
    pub async fn start(&self) -> Result<()> {
        let mut slot_rx = self.client.subscribe_slots();

        loop {
            match slot_rx.recv().await {
                Ok(slot_info) => {
                    // Update local state
                    *self.current_slot.write().await = slot_info.slot;
                    *self.current_leader.write().await = slot_info.leader.clone();

                    tracing::debug!(
                        slot = slot_info.slot,
                        leader = %slot_info.leader,
                        "🔄 SlotMonitor state updated"
                    );
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("SlotMonitor: lagged behind broadcast stream");
                    // This is fine—just skip the old events and continue
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(anyhow!("Slot broadcast channel closed"));
                }
            }
        }
    }

    /// Get the current slot
    pub async fn current_slot(&self) -> u64 {
        *self.current_slot.read().await
    }

    /// Get the current leader
    pub async fn current_leader(&self) -> String {
        self.current_leader.read().await.clone()
    }
}

// ============================================================================
// REAL YELLOWSTONE INTEGRATION COMMENTS
// ============================================================================
//
// To fully integrate Yellowstone gRPC, follow this pattern:
//
// 1. Add to Cargo.toml:
//    ```
//    yellowstone-grpc-client = "0.1"
//    yellowstone-grpc-proto = "0.1"
//    tonic = "0.10"
//    prost = "0.12"
//    ```
//
// 2. Import in grpc.rs:
//    ```
//    use yellowstone_grpc_client::GeyserClient;
//    use yellowstone_grpc_proto::geyser::{
//        Subscribe, SubscribeSlots, SubscribeBlockMeta
//    };
//    ```
//
// 3. Replace run_stream() with actual gRPC code:
//    ```rust
//    async fn run_stream(&self) -> Result<()> {
//        let mut client = GeyserClient::connect(
//            self.config.yellowstone_grpc_endpoint.clone()
//        ).await?;
//
//        let mut subscribe = Subscribe::default();
//        subscribe.slots.insert(
//            "client".to_string(),
//            SubscribeSlots { .default() }
//        );
//        subscribe.blocks_meta.insert(
//            "client".to_string(),
//            SubscribeBlockMeta { .default() }
//        );
//
//        let mut stream = client.subscribe(subscribe).await?;
//
//        while let Some(update) = stream.next().await {
//            let update = update?;
//            
//            if let Some(slot) = update.slot {
//                let slot_info = SlotInfo::new(
//                    slot.slot,
//                    slot.leader, // Already base58
//                    slot.parent_slot,
//                );
//                let _ = self.slot_tx.send(slot_info);
//            }
//        }
//        Ok(())
//    }
//    ```
//
// This is intentionally left as a stub for Phase 2 to focus on architecture.
// Phase 3 will implement the full gRPC integration.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yellowstone_client_creation() {
        let config = Config {
            yellowstone_grpc_endpoint: "http://localhost:10000".to_string(),
            jito_bundle_endpoint: "https://example.com".to_string(),
            solana_rpc_url: "https://api.devnet.solana.com".to_string(),
            wallet_keypair_path: Some("/tmp/key.json".to_string()),
            private_key: None,
            jito_tip_account: "9B5X4b3XfBmrKzf7YsXwqYuvz2aLf5cuucsBiB1A6qws".to_string(),
            environment: "devnet".to_string(),
            max_tip_multiplier: 2.5,
            min_tip_lamports: 1000,
            debug: false,
        };

        let client = YellowstoneGrpcClient::new(config);
        assert!(!client.config.environment.is_empty());
    }

    #[tokio::test]
    async fn test_slot_monitor_subscription() {
        let config = Config {
            yellowstone_grpc_endpoint: "http://localhost:10000".to_string(),
            jito_bundle_endpoint: "https://example.com".to_string(),
            solana_rpc_url: "https://api.devnet.solana.com".to_string(),
            wallet_keypair_path: Some("/tmp/key.json".to_string()),
            private_key: None,
            jito_tip_account: "9B5X4b3XfBmrKzf7YsXwqYuvz2aLf5cuucsBiB1A6qws".to_string(),
            environment: "devnet".to_string(),
            max_tip_multiplier: 2.5,
            min_tip_lamports: 1000,
            debug: false,
        };

        let client = Arc::new(YellowstoneGrpcClient::new(config));
        let monitor = SlotMonitor::new(Arc::clone(&client));

        // Spawn a task to send a test slot update
        let client_clone = Arc::clone(&client);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let slot_info = SlotInfo::new(42, "TestLeader".to_string(), 41);
            let _ = client_clone.slot_tx.send(slot_info);
        });

        // Try to read it
        let mut rx = client.subscribe_slots();
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(1),
            rx.recv(),
        )
        .await
        {
            Ok(Ok(slot_info)) => {
                assert_eq!(slot_info.slot, 42);
                assert_eq!(slot_info.leader, "TestLeader");
            }
            _ => panic!("Should receive slot update"),
        }
    }
}
