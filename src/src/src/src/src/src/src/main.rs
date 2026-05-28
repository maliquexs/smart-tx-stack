/// Smart Transaction Stack - Main entry point
/// 
/// Phase 2 initialization:
/// 1. Load configuration from .env
/// 2. Initialize tracing (structured logging)
/// 3. Connect to Yellowstone gRPC
/// 4. Start slot + leader monitoring
/// 5. Stay alive until ctrl-c

use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tracing_subscriber;

mod config;
mod grpc;
mod jito;
mod lifecycle;
mod ai_agent;
mod types;

use config::Config;
use grpc::{YellowstoneGrpcClient, SlotMonitor};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing before anything else
    // This makes all tracing::info! and tracing::debug! calls visible
    initialize_tracing();

    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("🚀 Smart Transaction Stack v0.1.0 (Phase 2)");
    tracing::info!("   Advanced Infrastructure Challenge – Superteam Nigeria");
    tracing::info!("═══════════════════════════════════════════════════════════");

    // Load configuration from .env file
    let config = Config::load()
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to load configuration");
            e
        })?;

    // Log configuration (hide sensitive keys)
    tracing::info!(
        environment = %config.environment,
        yellowstone = %config.yellowstone_grpc_endpoint,
        jito = %config.jito_bundle_endpoint,
        solana_rpc = %config.solana_rpc_url,
        debug = config.debug,
        "📋 Configuration loaded"
    );

    // Validate endpoints are well-formed
    config
        .validate_endpoints()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Configuration validation failed");
            e
        })?;

    tracing::info!("✅ All configuration validated");

    // Create Yellowstone gRPC client
    let grpc_client = Arc::new(YellowstoneGrpcClient::new(config.clone()));

    tracing::info!("🔌 Connecting to Yellowstone gRPC...");

    // Spawn the slot monitor task
    let monitor = SlotMonitor::new(Arc::clone(&grpc_client));
    let monitor_handle = {
        let monitor = monitor.clone();
        tokio::spawn(async move {
            if let Err(e) = monitor.start().await {
                tracing::error!(error = %e, "SlotMonitor encountered error");
            }
        })
    };

    // Spawn the gRPC connection task
    let grpc_handle = {
        let client = Arc::clone(&grpc_client);
        tokio::spawn(async move {
            if let Err(e) = client.start_monitoring().await {
                tracing::error!(error = %e, "Yellowstone monitoring error (expected on shutdown)");
            }
        })
    };

    tracing::info!("✅ Yellowstone gRPC listener started");
    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("👂 Listening for slot updates...");
    tracing::info!("Press Ctrl+C to stop");
    tracing::info!("═══════════════════════════════════════════════════════════");

    // Demonstrate reading from the monitor
    let monitor_clone = monitor.clone();
    let demo_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            let slot = monitor_clone.current_slot().await;
            let leader = monitor_clone.current_leader().await;
            tracing::info!(
                current_slot = slot,
                current_leader = %leader,
                "📊 Current state snapshot"
            );
        }
    });

    // Wait for ctrl-c signal
    signal::ctrl_c().await?;

    tracing::info!("🛑 Shutdown signal received");
    tracing::info!("Gracefully shutting down...");

    // Cancel all tasks
    grpc_handle.abort();
    monitor_handle.abort();
    demo_handle.abort();

    tracing::info!("✅ All tasks shut down");
    tracing::info!("goodbye 👋");

    Ok(())
}

/// Initialize structured logging with tracing
fn initialize_tracing() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();
}

// ============================================================================
// FUTURE EXTENSIONS (Phase 3+)
// ============================================================================
//
// Once Phase 2 (monitoring) is solid, we'll add:
//
// Phase 3 - Dynamic Tip Calculation:
// - Implement jito::DynamicTipCalculator
// - Query getTipAccounts endpoint
// - Calculate optimal tips based on mempool congestion
//
// Phase 4 - Bundle Building & Submission:
// - Implement jito::BundleBuilder
// - Create Jito SearcherAPI client
// - Submit bundles with dynamic tips
// - Track submission status
//
// Phase 5 - AI Agent & Failure Recovery:
// - Implement ai_agent::FailureClassifier
// - Implement ai_agent::AutonomousAgent
// - Decision tree: Retry? Hold? Resubmit with higher tip?
// - Log all reasoning in JSON for judges
//
// Phase 6 - Observability & Metrics:
// - Implement lifecycle::LifecycleTracker
// - Aggregate bundle stats
// - Dashboard integration
// - Performance metrics (latency, success rate, cost)
// ============================================================================
          
