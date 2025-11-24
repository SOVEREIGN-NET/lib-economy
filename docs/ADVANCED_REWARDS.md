# Advanced Rewards System

Comprehensive documentation of the ZHTP quality tier system, bonuses, and reward adjustments.

## Overview

The ZHTP rewards system goes beyond simple work-based compensation, implementing a sophisticated multi-tier quality system that rewards consistent, high-quality infrastructure provision.

## Table of Contents

1. [Infrastructure Rewards](#infrastructure-rewards)
2. [Quality Bonus Tiers](#quality-bonus-tiers)
3. [Reward Adjustments](#reward-adjustments)
4. [Consensus Statistics](#consensus-statistics)
5. [Complete Reward Calculator](#complete-reward-calculator)

## Infrastructure Rewards

### Base Reward Calculation

```rust
pub struct InfrastructureRewardEngine {
    pub bandwidth_rate: u64,     // 1 ZHTP per MB
    pub storage_rate: u64,       // 10 ZHTP per GB
    pub validation_rate: u64,    // 100 ZHTP per validation
}

impl InfrastructureRewardEngine {
    pub fn calculate_rewards(
        &self,
        work: &WorkMetrics,
    ) -> InfrastructureRewards {
        // 1. Base rewards
        let bandwidth_reward = (work.routing_work / 1_000_000) * self.bandwidth_rate;
        let storage_reward = (work.storage_work / 1_000_000_000) * self.storage_rate;
        let validation_reward = work.compute_work * self.validation_rate;
        
        let base_total = bandwidth_reward + storage_reward + validation_reward;
        
        // 2. Quality multiplier (0.5x to 2.0x)
        let quality_multiplier = 0.5 + (work.quality_score * 1.5);
        let quality_adjusted = (base_total as f64 * quality_multiplier) as u64;
        
        // 3. Uptime bonus (10 ZHTP per hour)
        let uptime_bonus = work.uptime_hours * 10;
        
        // 4. Consistency bonus (5% if uptime > 95%)
        let consistency_bonus = if work.uptime_hours >= 23 {  // ~95% of day
            (quality_adjusted as f64 * 0.05) as u64
        } else {
            0
        };
        
        let total = quality_adjusted + uptime_bonus + consistency_bonus;
        
        InfrastructureRewards {
            bandwidth_reward,
            storage_reward,
            validation_reward,
            quality_multiplier,
            uptime_bonus,
            consistency_bonus,
            total_reward: total,
        }
    }
}
```

### Reward Rate Constants

```rust
// From lib.rs
pub const ROUTING_REWARD_PER_MB: u64 = 1;           // 1 ZHTP/MB
pub const STORAGE_REWARD_PER_GB: u64 = 10;          // 10 ZHTP/GB
pub const COMPUTE_REWARD_PER_VALIDATION: u64 = 100; // 100 ZHTP/validation
pub const UPTIME_BONUS_RATE: u64 = 10;              // 10 ZHTP/hour
```

### Example Calculation

```rust
// High-quality node, 24 hours
let work = WorkMetrics {
    routing_work: 100_000_000,        // 100 MB
    storage_work: 50_000_000_000,     // 50 GB
    compute_work: 10,                 // 10 validations
    quality_score: 0.95,              // 95% quality
    uptime_hours: 24,                 // 24 hours
};

let rewards = engine.calculate_rewards(&work);

// Results:
// bandwidth_reward = 100 MB / 1 MB = 100 ZHTP
// storage_reward = 50 GB * 10 = 500 ZHTP
// validation_reward = 10 * 100 = 1,000 ZHTP
// base_total = 1,600 ZHTP
// quality_multiplier = 0.5 + (0.95 * 1.5) = 1.925
// quality_adjusted = 1,600 * 1.925 = 3,080 ZHTP
// uptime_bonus = 24 * 10 = 240 ZHTP
// consistency_bonus = 3,080 * 0.05 = 154 ZHTP
// total_reward = 3,474 ZHTP/day
```

## Quality Bonus Tiers

### Tier System

The quality bonus system rewards sustained high performance with progressively larger bonuses:

```rust
pub struct QualityBonusEngine {
    pub tiers: Vec<QualityTier>,
}

pub struct QualityTier {
    pub name: String,
    pub min_score: f64,
    pub bonus_percentage: f64,
    pub badge: String,
}
```

### Tier Definitions

| Tier | Badge | Min Score | Bonus | Requirements |
|------|-------|-----------|-------|--------------|
| Diamond | 💎 | 0.95 (95%) | +50% | Near-perfect quality |
| Platinum | 🏆 | 0.90 (90%) | +35% | Excellent quality |
| Gold | 🥇 | 0.85 (85%) | +20% | Very good quality |
| Silver | 🥈 | 0.75 (75%) | +10% | Good quality |
| Bronze | 🥉 | 0.60 (60%) | +5% | Acceptable quality |

### Quality Score Calculation

Quality score is a weighted combination of metrics:

```rust
pub fn calculate_quality_score(metrics: &QualityMetrics) -> f64 {
    (metrics.uptime_score * 0.4) +           // 40% weight
    (metrics.latency_score * 0.3) +          // 30% weight
    (metrics.reliability_score * 0.3)        // 30% weight
}

// Example scores:
// uptime_score = 0.98 (98% uptime)
// latency_score = 0.92 (low latency)
// reliability_score = 0.96 (high reliability)
// quality_score = (0.98 * 0.4) + (0.92 * 0.3) + (0.96 * 0.3)
//               = 0.392 + 0.276 + 0.288
//               = 0.956 (95.6%) → Diamond Tier
```

### Bonus Application

```rust
impl QualityBonusEngine {
    pub fn calculate_bonus(
        &self,
        base_reward: u64,
        quality_score: f64,
    ) -> QualityBonus {
        let tier = self.get_tier(quality_score);
        
        let bonus_amount = match tier {
            Some(t) => (base_reward as f64 * t.bonus_percentage) as u64,
            None => 0,
        };
        
        QualityBonus {
            base_reward,
            bonus_amount,
            total_reward: base_reward + bonus_amount,
            quality_score,
            tier: tier.cloned(),
        }
    }
}
```

### Example with Tiers

```rust
// Diamond tier node (95.6% quality)
base_reward = 3,000 ZHTP
quality_score = 0.956
tier = Diamond (50% bonus)

bonus_amount = 3,000 * 0.50 = 1,500 ZHTP
total_reward = 3,000 + 1,500 = 4,500 ZHTP

// Silver tier node (78% quality)
base_reward = 3,000 ZHTP
quality_score = 0.78
tier = Silver (10% bonus)

bonus_amount = 3,000 * 0.10 = 300 ZHTP
total_reward = 3,000 + 300 = 3,300 ZHTP
```

## Reward Adjustments

### Dynamic Adjustments

Rewards automatically adjust based on network conditions:

```rust
pub struct RewardAdjustmentEngine {
    pub participation_threshold: f64,  // 0.7 = 70%
    pub quality_threshold: f64,        // 0.8 = 80%
}

impl RewardAdjustmentEngine {
    /// Apply quality multiplier to base reward
    pub fn apply_quality_multiplier(
        &self,
        base_reward: u64,
        quality_metrics: &QualityMetrics,
    ) -> u64 {
        let quality_score = calculate_quality_score(quality_metrics);
        
        let multiplier = if quality_score >= 0.95 {
            1.5  // Excellent quality bonus
        } else if quality_score >= 0.85 {
            1.2  // Good quality bonus
        } else if quality_score >= 0.70 {
            1.0  // Standard reward
        } else if quality_score >= 0.50 {
            0.7  // Reduced reward for mediocre quality
        } else {
            0.4  // Significant penalty for poor quality
        };
        
        (base_reward as f64 * multiplier) as u64
    }
    
    /// Adjust rewards for network participation level
    pub fn adjust_for_participation(
        &self,
        base_reward: u64,
        network_participation_rate: f64,
    ) -> u64 {
        if network_participation_rate < self.participation_threshold {
            // Boost rewards to encourage participation
            (base_reward as f64 * 1.5) as u64
        } else if network_participation_rate > 0.95 {
            // Reduce rewards when network is saturated
            (base_reward as f64 * 0.8) as u64
        } else {
            base_reward
        }
    }
}
```

### Network Health Bonus

Additional bonus when network is healthy:

```rust
pub fn calculate_health_bonus(
    base_reward: u64,
    network_health: f64,  // 0.0-1.0
) -> u64 {
    if network_health > 0.9 {
        // 5% bonus for contributing to healthy network
        (base_reward as f64 * 0.05) as u64
    } else {
        0
    }
}
```

## Consensus Statistics

### Validator Performance Tracking

```rust
pub struct ConsensusStatistics {
    pub total_blocks: u64,
    pub validator_stats: HashMap<[u8; 32], ValidatorStats>,
}

pub struct ValidatorStats {
    pub blocks_proposed: u64,
    pub blocks_validated: u64,
    pub blocks_missed: u64,
    pub total_rewards: u64,
    pub average_validation_time_ms: u64,
    pub uptime_percentage: f64,
}

impl ConsensusStatistics {
    /// Calculate validator performance score
    pub fn calculate_performance_score(&self, validator: &[u8; 32]) -> f64 {
        if let Some(stats) = self.validator_stats.get(validator) {
            let proposal_rate = stats.blocks_proposed as f64 / self.total_blocks as f64;
            let miss_rate = stats.blocks_missed as f64 / stats.blocks_proposed.max(1) as f64;
            let speed_score = 1.0 - (stats.average_validation_time_ms.min(1000) as f64 / 1000.0);
            
            // Weighted score
            (proposal_rate * 0.4) +           // 40% - block production
            ((1.0 - miss_rate) * 0.4) +       // 40% - reliability
            (speed_score * 0.2)               // 20% - speed
        } else {
            0.0
        }
    }
}
```

### Validator Reward Calculation

```rust
pub struct ValidatorRewards {
    pub block_reward: u64,
    pub transaction_fees: u64,
    pub uptime_bonus: u64,
    pub performance_bonus: u64,
}

impl ValidatorRewards {
    pub fn calculate(
        blocks_produced: u64,
        transactions_included: u64,
        uptime_percentage: f64,
        missed_blocks: u64,
    ) -> Self {
        // Base block reward
        let block_reward = blocks_produced * 50_000; // 50K ZHTP per block
        
        // Transaction fee share (validators get 80% of network fees)
        let transaction_fees = transactions_included * 80; // ~80 ZHTP average
        
        // Uptime bonus (up to 20% of base reward)
        let uptime_bonus = (block_reward as f64 * uptime_percentage * 0.20) as u64;
        
        // Performance penalty for missed blocks
        let penalty = missed_blocks * 10_000;
        let performance_bonus = if penalty > block_reward {
            0
        } else {
            block_reward - penalty
        };
        
        ValidatorRewards {
            block_reward,
            transaction_fees,
            uptime_bonus,
            performance_bonus,
        }
    }
    
    pub fn total(&self) -> u64 {
        self.block_reward 
            + self.transaction_fees 
            + self.uptime_bonus 
            + self.performance_bonus
    }
}
```

## Complete Reward Calculator

### Unified Reward Calculation

```rust
pub struct RewardCalculator {
    pub model: EconomicModel,
    pub infrastructure_engine: InfrastructureRewardEngine,
    pub quality_engine: QualityBonusEngine,
    pub adjustment_engine: RewardAdjustmentEngine,
}

impl RewardCalculator {
    pub fn calculate_complete_reward(
        &self,
        work: &WorkMetrics,
        network_stats: &NetworkStats,
    ) -> Result<CompleteReward> {
        // 1. Base infrastructure rewards
        let infrastructure = self.infrastructure_engine.calculate_rewards(work);
        
        // 2. Quality tier bonus
        let quality = self.quality_engine.calculate_bonus(
            infrastructure.total_reward,
            work.quality_score,
        );
        
        // 3. Network participation adjustment
        let participation_rate = network_stats.active_nodes as f64 / 10000.0;
        let participation_adjusted = self.adjustment_engine.adjust_for_participation(
            quality.total_reward,
            participation_rate,
        );
        
        // 4. Network health bonus
        let health_bonus = if network_stats.calculate_health_score() > 0.9 {
            (participation_adjusted as f64 * 0.05) as u64  // 5% bonus
        } else {
            0
        };
        
        let final_reward = participation_adjusted + health_bonus;
        
        Ok(CompleteReward {
            base_infrastructure: infrastructure.total_reward,
            quality_bonus: quality.bonus_amount,
            quality_tier: quality.tier,
            participation_adjustment: participation_adjusted - quality.total_reward,
            health_bonus,
            total_reward: final_reward,
            work_breakdown: infrastructure,
        })
    }
}
```

### Complete Example

```rust
// Diamond-tier node on healthy network
let work = WorkMetrics {
    routing_work: 500_000_000,        // 500 MB
    storage_work: 100_000_000_000,    // 100 GB
    compute_work: 20,                 // 20 validations
    quality_score: 0.96,              // 96% quality (Diamond)
    uptime_hours: 24,                 // 24 hours
};

let network_stats = NetworkStats {
    active_nodes: 5000,
    network_congestion: 0.4,
    health_score: 0.92,               // Healthy network
    // ...
};

let complete = calculator.calculate_complete_reward(&work, &network_stats)?;

// Results:
// Base Infrastructure:
//   - Routing: 500 ZHTP
//   - Storage: 1,000 ZHTP
//   - Compute: 2,000 ZHTP
//   - Quality adjusted: ~7,000 ZHTP
//   - Uptime bonus: 240 ZHTP
//   - Consistency: 350 ZHTP
//   - Subtotal: 7,590 ZHTP
//
// Quality Bonus (Diamond 50%):
//   - Bonus: 3,795 ZHTP
//   - New total: 11,385 ZHTP
//
// Participation Adjustment:
//   - Rate: 50% (moderate)
//   - No adjustment: 11,385 ZHTP
//
// Health Bonus (5%):
//   - Bonus: 569 ZHTP
//
// FINAL TOTAL: 11,954 ZHTP/day
```

## ISP Bypass Rewards

Special rewards for replacing traditional ISPs:

```rust
pub fn calculate_isp_bypass(work: &IspBypassWork) -> Result<TokenReward> {
    // Constants from lib.rs
    const CONNECTIVITY_RATE: u64 = 100;  // 100 ZHTP/GB
    const MESH_RATE: u64 = 1;            // 1 ZHTP/MB
    const UPTIME_BONUS: u64 = 10;        // 10 ZHTP/hour
    
    // Bandwidth sharing: 100 ZHTP per GB
    let connectivity_reward = work.bandwidth_shared_gb * CONNECTIVITY_RATE;
    
    // Packet routing in mesh: 1 ZHTP per MB
    let mesh_reward = work.packets_routed_mb * MESH_RATE;
    
    // Uptime bonus
    let uptime_bonus = work.uptime_hours * UPTIME_BONUS;
    
    // Quality multiplier (0.8 to 1.2)
    let quality_multiplier = 0.8 + (work.connection_quality * 0.4);
    
    // Users served bonus (50 ZHTP per user)
    let users_bonus = work.users_served * 50;
    
    let base_total = connectivity_reward + mesh_reward + uptime_bonus + users_bonus;
    let total_reward = (base_total as f64 * quality_multiplier) as u64;
    
    Ok(TokenReward {
        routing_reward: mesh_reward,
        storage_reward: 0,
        compute_reward: 0,
        quality_bonus: users_bonus,
        uptime_bonus,
        total_reward,
    })
}
```

## Mesh Network Rewards

Additional rewards for mesh network participation:

```rust
pub struct MeshDiscoveryRewards {
    pub peer_discovery_bonus: u64,      // 100 ZHTP per peer
    pub route_optimization_bonus: u64,  // 50 ZHTP per route
    pub network_healing_bonus: u64,     // 5K ZHTP for healing
}

impl MeshDiscoveryRewards {
    pub fn calculate(
        peers_discovered: u64,
        routes_optimized: u64,
        network_partitions_healed: u64,
    ) -> Self {
        MeshDiscoveryRewards {
            peer_discovery_bonus: peers_discovered * 100,
            route_optimization_bonus: routes_optimized * 50,
            network_healing_bonus: network_partitions_healed * 5_000,
        }
    }
    
    pub fn total(&self) -> u64 {
        self.peer_discovery_bonus 
            + self.route_optimization_bonus 
            + self.network_healing_bonus
    }
}
```

## Summary

The ZHTP advanced rewards system provides:

- ✅ **Multi-dimensional rewards** - Bandwidth, storage, compute, uptime
- ✅ **Quality tier system** - Diamond to Bronze with 5-50% bonuses
- ✅ **Dynamic adjustments** - Network participation and health bonuses
- ✅ **Anti-Sybil protection** - Quality-based, not node-count based
- ✅ **Consensus rewards** - Performance-based validator compensation
- ✅ **ISP replacement** - Special rewards for connectivity providers
- ✅ **Mesh incentives** - Discovery, routing, and healing bonuses

This creates a sustainable ecosystem that rewards **genuine, high-quality infrastructure contribution** over gaming the system.
