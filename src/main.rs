use anyhow::Result;
use jamey_3::api::create_app;
use jamey_3::config::Config;
use jamey_3::db::init_db;
use jamey_3::mqtt::{MqttClient, handlers};
use jamey_3::conscience::ConscienceEngine;
use jamey_3::memory::MemorySystem;
use jamey_3::phoenix::{PhoenixVault, BackupScheduler};
use jamey_3::soul::SoulStorage;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::broadcast;
use tracing::{info, warn, error};
// UtcTime requires the "time" feature which is not enabled
// Using system time instead

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize telemetry (OpenTelemetry) if available, otherwise fall back to JSON logging
    if let Err(e) = jamey_3::telemetry::init_telemetry() {
        warn!("Failed to initialize telemetry (OpenTelemetry), falling back to JSON logging: {}", e);
        // Fallback to JSON-formatted tracing
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .with_target(true)
            .init();
    }

    info!("Starting Jamey 3.0 - General & Guardian");

    // Load configuration
    let config = match Config::from_env() {
        Ok(Some(cfg)) => {
            cfg.validate()?;
            info!("Configuration loaded successfully");
            
            if !cfg.core.openrouter_api_key.is_empty() {
                info!("OpenRouter API key loaded - LLM features available");
            } else {
                info!("OpenRouter API key not set - LLM features unavailable");
                info!("To enable LLM features, set OPENROUTER_API_KEY in your .env file");
            }
            
            if cfg.operational.dev_mode {
                info!("Running in development mode");
            }
            
            Some(cfg)
        }
        Ok(None) => {
            info!("Minimal configuration loaded - some features unavailable");
            None
        }
        Err(e) => {
            warn!("Configuration warning: {}", e);
            None
        }
    };

    // Initialize database
    let pool = init_db().await?;
    info!("Database initialized");

    // Initialize Soul system if enabled (needed before wiring to components)
    let soul = if let Some(_cfg) = config.as_ref().map(|c| &c.soul) {
        info!("Initializing Soul system...");
        let storage = SoulStorage::new(pool.clone());
        Some(Arc::new(storage))
    } else {
        info!("Soul system configuration not found");
        None
    };

    // Initialize core components with Soul integration
    let conscience_engine = ConscienceEngine::new();
    let conscience = if let Some(ref soul_storage) = soul {
        Arc::new(conscience_engine.with_soul_storage(Arc::clone(soul_storage)))
    } else {
        Arc::new(conscience_engine)
    };
    
    let memory_system = MemorySystem::new(
        config.as_ref()
            .and_then(|c| c.core.data_dir.as_ref())
            .map(|d| format!("{}/memory", d))
            .unwrap_or_else(|| "data/memory".to_string())
            .into()
    ).await?;
    let memory = if let Some(ref soul_storage) = soul {
        Arc::new(memory_system.with_soul_storage(Arc::clone(soul_storage)))
    } else {
        Arc::new(memory_system)
    };

    // Initialize Phoenix Vault if enabled
    let phoenix_vault = if let Some(cfg) = config.as_ref().and_then(|c| c.phoenix.as_ref()) {
        if cfg.enabled {
            // Convert encryption key string to [u8; 32]
            let mut key = [0u8; 32];
            let key_bytes = cfg.encryption_key.as_bytes();
            let len = key_bytes.len().min(32);
            key[..len].copy_from_slice(&key_bytes[..len]);
            
            let vault = Arc::new(PhoenixVault::new(
                cfg.backup_dir.clone().into(),
                key,
                true,
                cfg.max_backups.try_into().unwrap(),
            )?);

            // Start backup scheduler if interval configured
            if let Some(hours) = cfg.auto_backup_hours {
                let scheduler = BackupScheduler::new(vault.clone(), hours.into());
                tokio::spawn(async move {
                    scheduler.start().await;
                });
            }

            Some(vault)
        } else {
            None
        }
    } else {
        None
    };

    // Initialize MQTT client if configured
    let mqtt_client = if let Some(mqtt_cfg) = config.as_ref().and_then(|c| c.mqtt.clone()) {
        info!("Initializing MQTT client...");
        let client = Arc::new(MqttClient::new(mqtt_cfg).await?);
        
        // Set up MQTT subscriptions with component references
        setup_mqtt_subscriptions(
            &client,
            conscience.clone(),
            memory.clone(),
        ).await?;
        
        Some(client)
    } else {
        info!("MQTT configuration not found - MQTT features unavailable");
        None
    };

    // Create shutdown channel
    let (shutdown_tx, _) = broadcast::channel(1);
    let mut shutdown_rx = shutdown_tx.subscribe();

    // Create and run the Axum app
    let app = create_app(
        pool.clone(),
        mqtt_client.clone(),
        soul.clone(),
    ).await?;

    let host = config.as_ref()
        .map(|c| c.operational.host.as_str())
        .unwrap_or("0.0.0.0");
    let port = config.as_ref()
        .map(|c| c.operational.port)
        .unwrap_or(3000);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    info!("Server listening on http://{}:{}", host, port);

    // Run server with graceful shutdown
    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(e) = result {
                error!("Server error: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal");
            let _ = shutdown_tx.send(());
        }
    }

    // Graceful shutdown
    info!("Starting graceful shutdown...");

    // Wait for in-flight requests to complete (up to 30 seconds)
    tokio::select! {
        _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
            warn!("Shutdown timeout reached");
        }
        _ = async {
            shutdown_rx.recv()
        } => {
            info!("All requests completed");
        }
    }

    // Disconnect MQTT if connected
    if let Some(mqtt) = mqtt_client {
        info!("Disconnecting MQTT client...");
        if let Err(e) = mqtt.disconnect().await {
            warn!("Error disconnecting MQTT: {}", e);
        }
    }

    // Final backup if Phoenix Vault enabled
    if let Some(vault) = phoenix_vault {
        info!("Creating final backup before shutdown...");
        if let Err(e) = vault.create_backup().await {
            warn!("Error creating final backup: {}", e);
        }
    }

    // Close database connections
    info!("Closing database connections...");
    pool.close().await;

    info!("Shutdown complete");
    Ok(())
}

/// Set up MQTT subscriptions and handlers
async fn setup_mqtt_subscriptions(
    mqtt: &Arc<MqttClient>,
    conscience: Arc<ConscienceEngine>,
    memory: Arc<MemorySystem>,
) -> Result<()> {
    use jamey_3::mqtt::{
        ConscienceEvaluationRequest, MemoryStoreRequest, QoS,
    };

    let mqtt_clone = Arc::clone(mqtt);
    let conscience_clone = Arc::clone(&conscience);
    
    // Subscribe to conscience evaluation requests
    mqtt.subscribe_typed::<ConscienceEvaluationRequest, _>(
        "jamey/conscience/evaluate",
        QoS::AtLeastOnce,
        move |msg| {
            let mqtt = Arc::clone(&mqtt_clone);
            let conscience = Arc::clone(&conscience_clone);
            tokio::spawn(async move {
                handlers::handle_conscience_evaluation(
                    mqtt,
                    conscience,
                    msg.payload,
                ).await;
            });
        },
    ).await?;

    let mqtt_clone2 = Arc::clone(mqtt);
    let memory_clone = Arc::clone(&memory);
    
    // Subscribe to memory store requests
    mqtt.subscribe_typed::<MemoryStoreRequest, _>(
        "jamey/memory/store",
        QoS::AtLeastOnce,
        move |msg| {
            let mqtt = Arc::clone(&mqtt_clone2);
            let memory = Arc::clone(&memory_clone);
            tokio::spawn(async move {
                handlers::handle_memory_store(
                    mqtt,
                    memory,
                    msg.payload,
                ).await;
            });
        },
    ).await?;

    info!("MQTT subscriptions configured");
    Ok(())
}
