# Integration Guide

Complete guide for integrating lib-economy with the ZHTP ecosystem and external services.

## Overview

lib-economy integrates with multiple components of the ZHTP network and external services including REST APIs, blockchain recording, network metrics, identity systems, and monitoring infrastructure.

## ZHTP Ecosystem Integration

### Core Dependencies

```rust
// Cargo.toml
[dependencies]
lib-identity = { path = "../lib-identity" }
lib-blockchain = { path = "../lib-blockchain" }
lib-network = { path = "../lib-network" }
lib-crypto = { path = "../lib-crypto" }
```

### Integration Architecture

```
┌──────────────────────────────────────────┐
│        lib-economy (Economic Layer)       │
└────────┬─────────┬─────────┬─────────────┘
         │         │         │
    ┌────▼────┐┌──▼──────┐┌─▼─────────┐
    │lib-     ││lib-     ││lib-       │
    │identity ││blockchain││network    │
    └─────────┘└──────────┘└───────────┘
```

## Identity Integration

### Wallet-Identity Binding

Every wallet must be bound to a verified identity:

```rust
use lib_identity::ZhtpIdentity;
use lib_economy::wallets::MultiWalletManager;

// Create wallet manager bound to identity
pub fn create_wallet_for_identity(
    identity: &ZhtpIdentity,
) -> Result<MultiWalletManager> {
    // Verify identity is active
    if !identity.is_active() {
        return Err(anyhow!("Identity must be active"));
    }
    
    // Create wallet manager
    let mut manager = MultiWalletManager::new(identity.id.0);
    
    // Create default wallets
    manager.create_wallet("Primary", WalletType::Personal)?;
    manager.create_wallet("Rewards", WalletType::Rewards)?;
    
    Ok(manager)
}
```

### Identity Verification for UBI

```rust
pub fn register_for_ubi(
    identity: &ZhtpIdentity,
    treasury: &mut DaoTreasury,
) -> Result<()> {
    // Must be full citizen
    if identity.access_level != AccessLevel::FullCitizen {
        return Err(anyhow!("Must be verified citizen for UBI"));
    }
    
    // Register in treasury
    treasury.ubi_recipients += 1;
    
    Ok(())
}
```

## Blockchain Integration

### Transaction Recording

```rust
use lib_blockchain::Transaction as BlockchainTransaction;

pub struct BlockchainEconomicBridge {
    pub model: EconomicModel,
    pub treasury: DaoTreasury,
}

impl BlockchainEconomicBridge {
    /// Process block rewards for validators
    pub async fn process_block_rewards(
        &mut self,
        validator: [u8; 32],
        block_height: u64,
        transactions_included: u64,
        total_fees: u64,
    ) -> Result<u64> {
        // Base block reward: 50K ZHTP
        let block_reward = 50_000;
        
        // Transaction fee share (80% to validator, 20% to treasury)
        let validator_fee_share = (total_fees as f64 * 0.80) as u64;
        let treasury_fee_share = (total_fees as f64 * 0.20) as u64;
        
        // Performance bonus based on transaction inclusion
        let performance_bonus = if transactions_included > 1000 {
            5_000
        } else if transactions_included > 500 {
            2_500
        } else {
            0
        };
        
        // Total validator reward
        let total_reward = block_reward + validator_fee_share + performance_bonus;
        
        // Send treasury share to DAO
        self.treasury.receive_dao_fee(treasury_fee_share)?;
        
        info!("Block {}: Validator {} earned {} ZHTP", 
            block_height, hex::encode(validator), total_reward);
        
        Ok(total_reward)
    }
    
    /// Verify economic constraints on blockchain
    pub async fn verify_economic_constraints(
        &self,
        block_transactions: &[Transaction],
    ) -> Result<bool> {
        let mut total_fees = 0u64;
        
        for tx in block_transactions {
            // Verify fee calculation is correct
            let (expected_network, expected_dao, expected_total) = 
                self.model.calculate_fee(tx.tx_size, tx.amount, tx.priority);
            
            if tx.network_fee != expected_network 
                || tx.dao_fee != expected_dao 
                || tx.total_fee != expected_total {
                warn!("Transaction {} has incorrect fees", hex::encode(tx.hash()));
                return Ok(false);
            }
            
            total_fees += tx.dao_fee;
        }
        
        info!("Block economic constraints verified. DAO fees: {}", total_fees);
        Ok(true)
    }
}
```

## Network Integration

### Bandwidth Pricing

```rust
use lib_network::NetworkMetrics;

pub struct NetworkEconomicIntegration {
    pub bandwidth_pricing: BandwidthPricing,
}

impl NetworkEconomicIntegration {
    /// Calculate real-time bandwidth pricing
    pub fn calculate_bandwidth_price(
        &self,
        network_congestion: f64,
        quality_of_service: QosLevel,
    ) -> u64 {
        let base_rate = 100; // 100 ZHTP per GB
        
        let congestion_multiplier = if network_congestion > 0.8 {
            1.0 + (network_congestion - 0.8) * 10.0  // Surge pricing
        } else if network_congestion < 0.3 {
            0.5  // Discount for low congestion
        } else {
            1.0
        };
        
        let qos_multiplier = match quality_of_service {
            QosLevel::BestEffort => 1.0,
            QosLevel::Standard => 1.5,
            QosLevel::Premium => 2.5,
            QosLevel::Guaranteed => 5.0,
        };
        
        (base_rate as f64 * congestion_multiplier * qos_multiplier) as u64
    }
    
    /// Reward low-latency nodes
    pub fn calculate_latency_reward(
        &self,
        average_latency_ms: u64,
        packets_routed: u64,
    ) -> u64 {
        let latency_multiplier = if average_latency_ms < 10 {
            2.0  // Double reward for <10ms
        } else if average_latency_ms < 50 {
            1.5
        } else if average_latency_ms < 100 {
            1.0
        } else {
            0.5  // Penalty for high latency
        };
        
        let base_reward = packets_routed / 1000; // 1 ZHTP per 1000 packets
        (base_reward as f64 * latency_multiplier) as u64
    }
}
```

## REST API Integration

### ZHTP Native API Client

```rust
pub struct ZhtpApiIntegration {
    pub base_url: String,
    pub client: reqwest::Client,
}

impl ZhtpApiIntegration {
    pub fn new(network: NetworkType) -> Self {
        let base_url = match network {
            NetworkType::Mainnet => "https://api.zhtp.network".to_string(),
            NetworkType::Testnet => "https://testnet.zhtp.network".to_string(),
            NetworkType::Devnet => "http://localhost:8080".to_string(),
        };
        
        ZhtpApiIntegration {
            base_url,
            client: reqwest::Client::new(),
        }
    }
    
    /// Get transaction fee estimate
    pub async fn estimate_fee(
        &self,
        tx_size: u64,
        amount: u64,
        priority: Priority,
    ) -> Result<FeeEstimate> {
        let response = self.client
            .post(&format!("{}/transactions/estimate_fee", self.base_url))
            .json(&serde_json::json!({
                "tx_size": tx_size,
                "amount": amount,
                "priority": priority,
            }))
            .send()
            .await?;
        
        let estimate: FeeEstimate = response.json().await?;
        Ok(estimate)
    }
    
    /// Submit economic transaction
    pub async fn submit_transaction(
        &self,
        tx: &Transaction,
    ) -> Result<TransactionReceipt> {
        let response = self.client
            .post(&format!("{}/transactions/submit", self.base_url))
            .json(&tx)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Transaction submission failed: {}", response.status()));
        }
        
        let receipt: TransactionReceipt = response.json().await?;
        Ok(receipt)
    }
    
    /// Get network congestion metrics
    pub async fn get_network_congestion(&self) -> Result<f64> {
        let response = self.client
            .get(&format!("{}/mempool/status", self.base_url))
            .send()
            .await?;
        
        let status: MempoolStatus = response.json().await?;
        
        // Calculate congestion from pending transactions
        let congestion = (status.pending_count as f64 / status.capacity as f64)
            .min(1.0);
        
        Ok(congestion)
    }
}
```

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/transactions/estimate_fee` | POST | Estimate transaction fees |
| `/transactions/submit` | POST | Submit transaction |
| `/mempool/status` | GET | Get mempool status |
| `/economy/treasury` | GET | Get treasury statistics |
| `/economy/ubi` | GET | Get UBI information |
| `/economy/rewards/{address}` | GET | Get reward balance |
| `/economy/wallets/{address}` | GET | Get wallet information |

## Configuration Management

### Network-Specific Configurations

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicConfig {
    pub network: NetworkType,
    pub base_fee_rate: u64,
    pub dao_fee_percentage: u64,
    pub ubi_target: u64,
    pub block_reward: u64,
    pub inflation_rate: f64,
}

impl EconomicConfig {
    pub fn load_for_network(network: NetworkType) -> Result<Self> {
        match network {
            NetworkType::Mainnet => Ok(EconomicConfig {
                network: NetworkType::Mainnet,
                base_fee_rate: 100,
                dao_fee_percentage: 200,
                ubi_target: 1000,
                block_reward: 50_000,
                inflation_rate: 0.02,
            }),
            NetworkType::Testnet => Ok(EconomicConfig {
                network: NetworkType::Testnet,
                base_fee_rate: 10,
                dao_fee_percentage: 200,
                ubi_target: 100,
                block_reward: 5_000,
                inflation_rate: 0.05,
            }),
            NetworkType::Devnet => Ok(EconomicConfig {
                network: NetworkType::Devnet,
                base_fee_rate: 1,
                dao_fee_percentage: 100,
                ubi_target: 10,
                block_reward: 1_000,
                inflation_rate: 0.10,
            }),
        }
    }
    
    pub fn apply_to_model(&self, model: &mut EconomicModel) {
        model.base_fee_rate = self.base_fee_rate;
        model.dao_fee_percentage = self.dao_fee_percentage;
        model.inflation_rate = self.inflation_rate;
    }
}
```

## Monitoring Integration

### Prometheus Metrics

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct EconomicMetrics {
    pub total_transactions: AtomicU64,
    pub total_fees_collected: AtomicU64,
    pub total_rewards_distributed: AtomicU64,
    pub total_ubi_distributed: AtomicU64,
    pub active_wallets: AtomicU64,
    pub treasury_balance: AtomicU64,
}

impl EconomicMetrics {
    /// Export metrics in Prometheus format
    pub fn to_prometheus(&self) -> String {
        format!(
            "# HELP zhtp_total_transactions Total number of economic transactions\\n\
             # TYPE zhtp_total_transactions counter\\n\
             zhtp_total_transactions {}\\n\
             \\n\
             # HELP zhtp_total_fees_collected Total fees collected (ZHTP)\\n\
             # TYPE zhtp_total_fees_collected counter\\n\
             zhtp_total_fees_collected {}\\n\
             \\n\
             # HELP zhtp_treasury_balance Treasury balance (ZHTP)\\n\
             # TYPE zhtp_treasury_balance gauge\\n\
             zhtp_treasury_balance {}\\n",
            self.total_transactions.load(Ordering::Relaxed),
            self.total_fees_collected.load(Ordering::Relaxed),
            self.treasury_balance.load(Ordering::Relaxed),
        )
    }
    
    /// Health check endpoint response
    pub fn health_status(&self) -> HealthStatus {
        let treasury_balance = self.treasury_balance.load(Ordering::Relaxed);
        let active_wallets = self.active_wallets.load(Ordering::Relaxed);
        
        HealthStatus {
            status: if treasury_balance > 0 && active_wallets > 0 {
                "healthy"
            } else {
                "degraded"
            }.to_string(),
            treasury_health: treasury_balance > 1_000_000,
            wallet_activity: active_wallets > 100,
        }
    }
}
```

## Mobile Integration

### iOS (Swift) Bindings

```rust
#[cfg(target_os = "ios")]
use swift_bridge::*;

#[cfg(target_os = "ios")]
#[swift_bridge::bridge]
mod mobile_ffi {
    extern "Rust" {
        type EconomicModel;
        
        #[swift_bridge(init)]
        fn new() -> EconomicModel;
        
        fn calculate_fee(
            &self,
            tx_size: u64,
            amount: u64,
            priority: u8,
        ) -> (u64, u64, u64);
    }
}
```

### Android (JNI) Bindings

```rust
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_network_zhtp_economy_EconomicModel_calculateFee(
    env: JNIEnv,
    _class: jni::objects::JClass,
    tx_size: jni::sys::jlong,
    amount: jni::sys::jlong,
    priority: jni::sys::jint,
) -> jni::objects::JObject {
    let model = EconomicModel::new();
    
    let priority = match priority {
        0 => Priority::Low,
        1 => Priority::Normal,
        2 => Priority::High,
        _ => Priority::Urgent,
    };
    
    let (network_fee, dao_fee, total_fee) = model.calculate_fee(
        tx_size as u64,
        amount as u64,
        priority,
    );
    
    let result = format!(
        r#"{{"network_fee": {}, "dao_fee": {}, "total_fee": {}}}"#,
        network_fee, dao_fee, total_fee
    );
    
    env.new_string(result)
        .expect("Failed to create Java string")
        .into()
}
```

## WASM Integration

### Web Bindings

```rust
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmEconomicApi {
    model: EconomicModel,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmEconomicApi {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmEconomicApi {
        WasmEconomicApi {
            model: EconomicModel::new(),
        }
    }
    
    #[wasm_bindgen]
    pub fn calculate_fee(
        &self,
        tx_size: u64,
        amount: u64,
        priority: u8,
    ) -> JsValue {
        let priority = match priority {
            0 => Priority::Low,
            1 => Priority::Normal,
            2 => Priority::High,
            _ => Priority::Urgent,
        };
        
        let (network_fee, dao_fee, total_fee) = 
            self.model.calculate_fee(tx_size, amount, priority);
        
        serde_wasm_bindgen::to_value(&serde_json::json!({
            "network_fee": network_fee,
            "dao_fee": dao_fee,
            "total_fee": total_fee,
        })).unwrap()
    }
}
```

## Security Integration

### Audit Logging

```rust
pub struct EconomicAuditLogger {
    log_file: PathBuf,
}

impl EconomicAuditLogger {
    /// Audit transaction processing
    pub fn audit_transaction(
        &self,
        tx: &Transaction,
        result: TransactionResult,
    ) -> Result<()> {
        let entry = AuditEntry {
            timestamp: chrono::Utc::now(),
            event_type: "TRANSACTION_PROCESSED".to_string(),
            severity: AuditSeverity::Info,
            details: serde_json::json!({
                "tx_hash": hex::encode(tx.hash()),
                "from": hex::encode(tx.from),
                "to": hex::encode(tx.to),
                "amount": tx.amount,
                "fee": tx.total_fee,
                "result": format!("{:?}", result),
            }),
        };
        
        self.write_entry(&entry)?;
        Ok(())
    }
    
    /// Audit suspicious activity
    pub fn audit_suspicious_activity(
        &self,
        wallet: [u8; 32],
        reason: &str,
    ) -> Result<()> {
        let entry = AuditEntry {
            timestamp: chrono::Utc::now(),
            event_type: "SUSPICIOUS_ACTIVITY".to_string(),
            severity: AuditSeverity::Warning,
            details: serde_json::json!({
                "wallet": hex::encode(wallet),
                "reason": reason,
            }),
        };
        
        self.write_entry(&entry)?;
        warn!("Suspicious economic activity: {}", reason);
        Ok(())
    }
}
```

## Complete Integration Example

```rust
/// Complete ZHTP economic system initialization
pub async fn initialize_zhtp_economy(
    network: NetworkType,
) -> Result<ZhtpEconomicSystem> {
    // 1. Load configuration
    let config = EconomicConfig::load_for_network(network)?;
    
    // 2. Initialize economic model
    let mut model = EconomicModel::new();
    config.apply_to_model(&mut model);
    
    // 3. Setup API client
    let api_client = ZhtpApiIntegration::new(network);
    
    // 4. Initialize metrics
    let metrics = EconomicMetrics::new();
    
    // 5. Setup audit logging
    let audit_logger = EconomicAuditLogger::new(PathBuf::from("./audit_logs"))?;
    
    // 6. Initialize blockchain bridge
    let blockchain_bridge = BlockchainEconomicBridge {
        model: model.clone(),
        treasury: DaoTreasury::new(),
    };
    
    // 7. Setup network integration
    let network_integration = NetworkEconomicIntegration::new();
    
    Ok(ZhtpEconomicSystem {
        model,
        api_client,
        metrics,
        audit_logger,
        blockchain_bridge,
        network_integration,
        config,
    })
}
```

## Summary

lib-economy integrates seamlessly with:

- ✅ **lib-identity** - Wallet-identity binding, UBI eligibility
- ✅ **lib-blockchain** - Transaction recording, block rewards
- ✅ **lib-network** - Bandwidth pricing, latency rewards
- ✅ **REST API** - ZHTP Native API for fee estimation, transaction submission
- ✅ **Prometheus** - Metrics export for monitoring
- ✅ **Mobile** - iOS/Android native bindings
- ✅ **Web** - WASM bindings for browser integration
- ✅ **Audit** - Comprehensive security logging

This enables full-stack economic functionality across the entire ZHTP ecosystem.
