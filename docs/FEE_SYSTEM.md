# Fee System

Complete documentation of the ZHTP fee calculation, processing, and distribution system.

## Overview

The ZHTP fee system implements a transparent, fair fee structure with mandatory DAO contributions that fund Universal Basic Income (UBI), welfare programs, and network development.

## Fee Structure

### Components

Every transaction incurs two types of fees:

1. **Network Fee** - Pays for transaction processing
2. **DAO Fee** - Mandatory 2% fee for welfare programs

```rust
total_fee = network_fee + dao_fee
dao_fee = network_fee × 0.02  // 2% of network fee
```

### Base Fee Calculation

```rust
pub fn calculate_fee(
    &self,
    tx_size: u64,      // Transaction size in bytes
    amount: u64,       // Transaction amount
    priority: Priority // Low, Normal, High, Urgent
) -> (u64, u64, u64) { // Returns (network_fee, dao_fee, total)
    
    // Step 1: Size factor (larger transactions cost more)
    let size_factor = (tx_size as f64 / 1000.0).max(1.0);
    
    // Step 2: Amount factor (larger amounts cost more)
    let amount_factor = (amount as f64 / 1_000_000.0).max(0.1);
    
    // Step 3: Base fee calculation
    let base_fee = (self.base_fee_rate as f64 
        * size_factor 
        * amount_factor) as u64;
    
    // Step 4: Priority multiplier
    let priority_multiplier = match priority {
        Priority::Low => 0.5,     // 50% discount
        Priority::Normal => 1.0,  // Standard rate
        Priority::High => 2.0,    // 2x priority
        Priority::Urgent => 5.0,  // 5x priority
    };
    
    // Step 5: Calculate network fee
    let network_fee = (base_fee as f64 * priority_multiplier) as u64;
    
    // Step 6: Calculate DAO fee (2%)
    let dao_fee = (network_fee as f64 * 0.02) as u64;
    
    // Step 7: Total fee
    let total_fee = network_fee + dao_fee;
    
    (network_fee, dao_fee, total_fee)
}
```

### Example Calculations

#### Small Transaction
```rust
// 1000 ZHTP, 250 bytes, Normal priority
tx_size = 250
amount = 1000
priority = Normal

size_factor = 250 / 1000 = 0.25 → 1.0 (minimum)
amount_factor = 1000 / 1_000_000 = 0.001 → 0.1 (minimum)
base_fee = 100 * 1.0 * 0.1 = 10
network_fee = 10 * 1.0 = 10 ZHTP
dao_fee = 10 * 0.02 = 0.2 ZHTP
total_fee = 10.2 ZHTP
```

#### Large Transaction
```rust
// 1,000,000 ZHTP, 1000 bytes, High priority
tx_size = 1000
amount = 1_000_000
priority = High

size_factor = 1000 / 1000 = 1.0
amount_factor = 1_000_000 / 1_000_000 = 1.0
base_fee = 100 * 1.0 * 1.0 = 100
network_fee = 100 * 2.0 = 200 ZHTP
dao_fee = 200 * 0.02 = 4 ZHTP
total_fee = 204 ZHTP
```

## Priority Fees

### Priority Levels

| Priority | Multiplier | Confirmation Time | Use Case |
|----------|-----------|-------------------|----------|
| Low | 0.5x | ~5 minutes | Non-urgent transfers |
| Normal | 1.0x | ~1 minute | Standard transactions |
| High | 2.0x | ~15 seconds | Time-sensitive payments |
| Urgent | 5.0x | ~3 seconds | Critical operations |

### Dynamic Priority Pricing

When network congestion is high, priority fees automatically adjust:

```rust
pub fn calculate_priority_fee(
    &self,
    base_fee: u64,
    priority: Priority,
    network_congestion: f64,  // 0.0 - 1.0
) -> u64 {
    let base_multiplier = match priority {
        Priority::Low => 0.5,
        Priority::Normal => 1.0,
        Priority::High => 2.0,
        Priority::Urgent => 5.0,
    };
    
    // Surge pricing when congestion > 80%
    let surge_multiplier = if network_congestion > 0.8 {
        1.0 + (network_congestion - 0.8) * 10.0  // Up to 3x
    } else {
        1.0
    };
    
    let final_multiplier = base_multiplier * surge_multiplier;
    
    (base_fee as f64 * final_multiplier) as u64
}
```

### Confirmation Time Estimation

```rust
pub fn estimate_confirmation_time(
    &self,
    priority: Priority,
    network_congestion: f64,
) -> Duration {
    let base_time = match priority {
        Priority::Low => 300,      // 5 minutes
        Priority::Normal => 60,    // 1 minute
        Priority::High => 15,      // 15 seconds
        Priority::Urgent => 3,     // 3 seconds
    };
    
    // Longer during congestion
    let congestion_multiplier = 1.0 + network_congestion;
    let estimated_seconds = (base_time as f64 * congestion_multiplier) as u64;
    
    Duration::from_secs(estimated_seconds)
}
```

## Fee Exemptions

### Fee-Free Transaction Types

Certain transaction types are **completely fee-free**:

```rust
pub enum TransactionType {
    Payment,                  // Regular payment (fees apply)
    Reward,                   // Infrastructure reward (fee-free) ✅
    UbiDistribution,          // UBI payment (fee-free) ✅
    WelfareDistribution,      // Welfare payment (fee-free) ✅
    Staking,                  // Staking operation (reduced fees)
    InfrastructureService,    // Service payment (fees apply)
}
```

### Reduced Fees

Staking operations have reduced fees:

```rust
if tx.tx_type == TransactionType::Staking {
    network_fee = network_fee / 2;  // 50% discount
}
```

## Fee Processing & Distribution

### Distribution Breakdown

Network fees are distributed as follows:

```
Network Fee (100%)
├─ Validators (80%)  ──▶ Block producers & validators
└─ Treasury (20%)    ──▶ Additional DAO funding

DAO Fee (100%)
└─ Treasury (100%)   ──▶ UBI/Welfare/Development
```

### Fee Processing Flow

```rust
pub struct FeeProcessor {
    pub treasury: DaoTreasury,
    pub validator_pool: u64,
}

impl FeeProcessor {
    pub fn process_transaction_fees(
        &mut self,
        tx: &Transaction,
    ) -> Result<FeeDistribution> {
        // DAO fee → 100% to treasury
        self.treasury.receive_dao_fee(tx.dao_fee)?;
        
        // Network fee → 80% validators, 20% treasury
        let validator_share = (tx.network_fee as f64 * 0.80) as u64;
        let treasury_share = (tx.network_fee as f64 * 0.20) as u64;
        
        self.validator_pool += validator_share;
        self.treasury.receive_dao_fee(treasury_share)?;
        
        Ok(FeeDistribution {
            dao_fee_to_treasury: tx.dao_fee,
            network_fee_to_validators: validator_share,
            network_fee_to_treasury: treasury_share,
            total_to_treasury: tx.dao_fee + treasury_share,
            total_to_validators: validator_share,
        })
    }
}
```

### Validator Fee Distribution

Validators share the validator pool proportionally to blocks produced:

```rust
pub async fn distribute_validator_fees(
    &mut self,
    validators: &[([u8; 32], u64)], // (address, blocks_produced)
) -> Result<Vec<ValidatorPayment>> {
    let total_blocks: u64 = validators.iter()
        .map(|(_, blocks)| blocks)
        .sum();
    
    if total_blocks == 0 {
        return Ok(Vec::new());
    }
    
    let mut payments = Vec::new();
    
    for (validator, blocks) in validators {
        let share = (self.validator_pool as f64 
            * (*blocks as f64 / total_blocks as f64)) as u64;
        
        payments.push(ValidatorPayment {
            validator: *validator,
            amount: share,
            blocks_produced: *blocks,
        });
    }
    
    // Reset pool after distribution
    self.validator_pool = 0;
    
    Ok(payments)
}
```

## DAO Fee Proofs

### Cryptographic Proof Generation

Every transaction includes a cryptographic proof that the DAO fee was calculated correctly:

```rust
pub struct DaoFeeProofGenerator {
    pub proof_version: u8,
}

impl DaoFeeProofGenerator {
    pub fn generate_proof(&self, tx: &Transaction) -> Vec<u8> {
        use blake3::Hasher;
        
        let mut hasher = Hasher::new();
        
        // Include transaction details
        hasher.update(&tx.from);
        hasher.update(&tx.to);
        hasher.update(&tx.amount.to_le_bytes());
        hasher.update(&tx.dao_fee.to_le_bytes());
        hasher.update(&tx.timestamp.to_le_bytes());
        hasher.update(&[self.proof_version]);
        
        // Generate proof
        let hash = hasher.finalize();
        hash.as_bytes().to_vec()
    }
    
    pub fn verify_proof(
        &self,
        tx: &Transaction,
        proof: &[u8],
    ) -> bool {
        let expected_proof = self.generate_proof(tx);
        expected_proof == proof
    }
}
```

### Proof in Transaction

```rust
pub struct Transaction {
    pub from: [u8; 32],
    pub to: [u8; 32],
    pub amount: u64,
    pub network_fee: u64,
    pub dao_fee: u64,
    pub total_fee: u64,
    pub dao_fee_proof: Vec<u8>,  // ✅ Cryptographic proof
    // ... other fields
}
```

## Transaction Validation

### Comprehensive Validation

```rust
pub struct TransactionValidator {
    pub min_amount: u64,
    pub max_amount: u64,
    pub require_dao_fee: bool,
}

impl TransactionValidator {
    pub fn validate(&self, tx: &Transaction) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // 1. Amount validation
        if tx.amount < self.min_amount {
            errors.push(format!("Amount below minimum: {}", tx.amount));
        }
        
        if tx.amount > self.max_amount {
            errors.push(format!("Amount exceeds maximum: {}", tx.amount));
        }
        
        // 2. Fee validation
        if self.require_dao_fee && tx.dao_fee == 0 {
            match tx.tx_type {
                TransactionType::Reward |
                TransactionType::UbiDistribution |
                TransactionType::WelfareDistribution => {
                    // Fee-exempt transactions
                }
                _ => {
                    errors.push("DAO fee required but not present".to_string());
                }
            }
        }
        
        // 3. Fee calculation verification
        let model = EconomicModel::new();
        let (expected_network, expected_dao, _) = model.calculate_fee(
            tx.tx_size,
            tx.amount,
            tx.priority,
        );
        
        if tx.network_fee != expected_network {
            warnings.push(format!(
                "Network fee mismatch: expected {}, got {}", 
                expected_network, tx.network_fee
            ));
        }
        
        if tx.dao_fee != expected_dao && self.require_dao_fee {
            warnings.push(format!(
                "DAO fee mismatch: expected {}, got {}", 
                expected_dao, tx.dao_fee
            ));
        }
        
        // 4. Proof verification
        let proof_gen = DaoFeeProofGenerator::new();
        if !proof_gen.verify_proof(tx, &tx.dao_fee_proof) {
            errors.push("Invalid DAO fee proof".to_string());
        }
        
        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
}
```

## Fee Economics

### Treasury Impact

```rust
// Example: 1000 daily transactions
daily_transactions = 1000
avg_network_fee = 100 ZHTP
avg_dao_fee = 2 ZHTP

// Daily DAO fee collection
daily_dao_fees = 1000 * 2 = 2,000 ZHTP

// Monthly DAO fee collection
monthly_dao_fees = 2,000 * 30 = 60,000 ZHTP

// Treasury allocation (40/30/30)
ubi_fund = 60,000 * 0.40 = 24,000 ZHTP
welfare_fund = 60,000 * 0.30 = 18,000 ZHTP
development_fund = 60,000 * 0.30 = 18,000 ZHTP

// UBI per citizen (1000 citizens)
ubi_per_citizen = 24,000 / 1000 = 24 ZHTP/month
```

### Fee Constants

```rust
// From lib.rs
pub const DEFAULT_DAO_FEE_RATE: u64 = 200;  // 2% (200 basis points)
pub const MIN_TRANSACTION_FEE: u64 = 10;    // 10 ZHTP minimum
pub const VALIDATOR_FEE_SHARE: f64 = 0.80;   // 80% to validators
```

## Anti-Speculation Mechanisms

### Progressive Trading Fees

To discourage day trading and speculation:

```rust
pub fn calculate_trading_fee(
    &self,
    trade_count_today: u64,
    amount: u64,
) -> u64 {
    let fee_rate = match trade_count_today {
        0..=5 => 0.01,      // 1%
        6..=10 => 0.02,     // 2%
        11..=20 => 0.05,    // 5%
        21..=50 => 0.10,    // 10%
        _ => 0.20,          // 20% (heavy discouragement)
    };
    
    (amount as f64 * fee_rate) as u64
}
```

### Trading Cooldowns

```rust
pub const TRADING_COOLDOWN_SECONDS: u64 = 300;  // 5 minutes
pub const MAX_DAILY_TRADES: u64 = 20;

pub fn enforce_cooldown(
    &self,
    last_trade_time: u64,
) -> Result<()> {
    let elapsed = current_timestamp() - last_trade_time;
    
    if elapsed < TRADING_COOLDOWN_SECONDS {
        let wait_time = TRADING_COOLDOWN_SECONDS - elapsed;
        return Err(anyhow!(
            "Trading cooldown active. Wait {} seconds",
            wait_time
        ));
    }
    
    Ok(())
}
```

## Fee Optimization Tips

### For Users

1. **Choose appropriate priority**
   - Use Low priority for non-urgent transfers (50% discount)
   - Use Normal for standard operations
   - Reserve High/Urgent for critical transactions

2. **Avoid network congestion**
   - Check mempool status before sending
   - Send during low-activity periods
   - Use fee estimation API

3. **Batch operations**
   - Combine multiple payments when possible
   - Use multi-send transactions

### For Validators

1. **Transaction selection**
   - Prioritize High/Urgent transactions
   - Fill blocks efficiently
   - Balance fee optimization with throughput

2. **Fee pool management**
   - Track validator pool balance
   - Claim rewards regularly
   - Monitor fee distribution

## API Integration

### Fee Estimation API

```rust
// GET /api/v1/fees/estimate
{
    "tx_size": 250,
    "amount": 1000,
    "priority": "Normal"
}

// Response
{
    "network_fee": 10,
    "dao_fee": 0.2,
    "total_fee": 10.2,
    "estimated_confirmation_time": 60
}
```

### Mempool Status API

```rust
// GET /api/v1/mempool/status
{
    "pending_count": 150,
    "capacity": 10000,
    "avg_fee": 100,
    "min_fee": 10,
    "max_fee": 500,
    "congestion": 0.015
}
```

## Summary

The ZHTP fee system implements:

- ✅ **Transparent fee calculation** - Clear formulas, no hidden costs
- ✅ **Mandatory welfare funding** - 2% DAO fee funds UBI and welfare
- ✅ **Priority-based processing** - Pay more for faster confirmation
- ✅ **Dynamic surge pricing** - Automatic congestion adjustment
- ✅ **Fee exemptions** - UBI, welfare, and rewards are fee-free
- ✅ **Cryptographic proofs** - Verifiable fee payments
- ✅ **Fair distribution** - 80% validators, 20% treasury
- ✅ **Anti-speculation** - Progressive fees and cooldowns

The system balances network sustainability with user affordability while ensuring continuous funding for welfare programs.
