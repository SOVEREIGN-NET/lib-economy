//! ISP bypass reward calculation and management
//! 
//! Implements comprehensive reward systems for nodes that provide ISP bypass services,
//! including bandwidth sharing, mesh routing, and cost savings tracking.
//! 
//! Integrates with lib-network and lib-consensus packages for real-time network data.

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::{TokenReward, EconomicModel};
use crate::types::{IspBypassWork, NetworkStats, WorkMetrics};
use crate::wallets::WalletBalance;
use crate::infrastructure_rewards::InfrastructureRewards;
use crate::quality_bonuses::QualityBonus;
use crate::network_participation::NetworkParticipationRewards;
use crate::distribution::RewardDistribution;
use crate::wasm::logging::info;

// External package imports for real implementations
use crate::network_types::{
    get_mesh_status, get_network_statistics, get_bandwidth_statistics, get_active_peer_count,
    MeshStatus, BandwidthStatistics, CongestionLevel
};
use crate::rewards::{RewardCalculator, ValidatorReward};
use lib_blockchain::{get_blockchain_health, get_current_block_height};

/// Comprehensive ISP bypass reward manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IspBypassRewardManager {
    /// Current ISP bypass work being tracked
    pub current_work: IspBypassWork,
    /// Historical performance metrics
    pub performance_history: Vec<IspBypassPerformanceRecord>,
    /// Accumulated cost savings provided to users
    pub total_cost_savings: u64,
    /// Number of users served
    pub users_served_total: u64,
    /// Quality score history for trend analysis
    pub quality_history: Vec<f64>,
    /// Uptime tracking for reliability bonuses
    pub uptime_stats: UptimeStats,
    /// Bandwidth utilization metrics
    pub bandwidth_metrics: BandwidthUtilization,
    /// Geographic coverage data
    pub coverage_metrics: CoverageMetrics,
    /// Anti-Sybil detection metrics
    pub authenticity_score: f64,
}

/// Performance record for historical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IspBypassPerformanceRecord {
    /// Timestamp of the record
    pub timestamp: u64,
    /// Work performed in this period
    pub work_performed: IspBypassWork,
    /// Reward earned in this period
    pub reward_earned: u64,
    /// Quality score achieved
    pub quality_score: f64,
    /// Uptime percentage for this period
    pub uptime_percentage: f64,
    /// Users served in this period
    pub unique_users_served: u32,
}

/// Uptime statistics for reliability tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeStats {
    /// Total uptime in hours since start
    pub total_uptime_hours: u64,
    /// Current session start time
    pub session_start: u64,
    /// Consecutive uptime hours
    pub consecutive_uptime: u64,
    /// Best uptime streak
    pub best_uptime_streak: u64,
    /// Downtime incidents
    pub downtime_incidents: u32,
}

/// Bandwidth utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthUtilization {
    /// Total bandwidth shared in GB
    pub total_bandwidth_gb: u64,
    /// Peak bandwidth capacity in Mbps
    pub peak_capacity_mbps: u64,
    /// Average utilization percentage
    pub avg_utilization: f64,
    /// Efficiency score (successful transfers / total attempts)
    pub efficiency_score: f64,
    /// Quality of service maintained
    pub qos_score: f64,
}

/// Geographic coverage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageMetrics {
    /// Estimated coverage area in km²
    pub coverage_area_km2: f64,
    /// Population density served
    pub population_density: f64,
    /// Rural vs urban service ratio
    pub rural_urban_ratio: f64,
    /// Connectivity gap filled
    pub connectivity_gap_filled: f64,
}

impl IspBypassRewardManager {
    /// Create new ISP bypass reward manager
    pub fn new() -> Self {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            current_work: IspBypassWork::new(),
            performance_history: Vec::new(),
            total_cost_savings: 0,
            users_served_total: 0,
            quality_history: Vec::new(),
            uptime_stats: UptimeStats {
                total_uptime_hours: 0,
                session_start: current_time,
                consecutive_uptime: 0,
                best_uptime_streak: 0,
                downtime_incidents: 0,
            },
            bandwidth_metrics: BandwidthUtilization {
                total_bandwidth_gb: 0,
                peak_capacity_mbps: 0,
                avg_utilization: 0.0,
                efficiency_score: 0.0,
                qos_score: 0.0,
            },
            coverage_metrics: CoverageMetrics {
                coverage_area_km2: 0.0,
                population_density: 0.0,
                rural_urban_ratio: 0.0,
                connectivity_gap_filled: 0.0,
            },
            authenticity_score: 1.0,
        }
    }

    /// Record ISP bypass work and update metrics using real network data
    pub async fn record_work(&mut self, work: IspBypassWork) -> Result<()> {
        // Get real-time network statistics from lib-network
        let mesh_status = get_mesh_status().await?;
        let bandwidth_stats = get_bandwidth_statistics().await?;
        let network_stats = get_network_statistics().await?;
        let peer_count = get_active_peer_count().await?;

        // Validate work against real network data
        self.validate_work_against_network(&work, &mesh_status, &bandwidth_stats).await?;

        self.record_work_internal(work, &mesh_status, &bandwidth_stats, peer_count).await
    }

    /// Internal method for recording work (used by public method and tests)
    async fn record_work_internal(&mut self, work: IspBypassWork, mesh_status: &MeshStatus, bandwidth_stats: &BandwidthStatistics, peer_count: i32) -> Result<()> {
        // Add work to current tracking
        self.current_work.add_bandwidth_shared(work.bandwidth_shared_gb);
        self.current_work.add_packets_routed(work.packets_routed_mb);
        self.current_work.update_connection_quality(work.connection_quality);
        self.current_work.add_users_served(work.users_served);
        self.current_work.add_cost_savings(work.cost_savings_provided);
        self.current_work.uptime_hours += work.uptime_hours;

        // Update aggregated metrics with real network context
        self.total_cost_savings += work.cost_savings_provided;
        self.users_served_total += work.users_served;
        
        // Update quality history with network-validated quality
        let validated_quality = self.calculate_validated_quality(&work, mesh_status).await?;
        self.quality_history.push(validated_quality);
        if self.quality_history.len() > 100 {
            self.quality_history.remove(0); // Keep last 100 entries
        }

        // Update uptime statistics based on mesh connectivity
        let network_uptime = mesh_status.connectivity_percentage / 100.0;
        let adjusted_uptime = work.uptime_hours as f64 * network_uptime;
        self.uptime_stats.total_uptime_hours += adjusted_uptime as u64;
        self.uptime_stats.consecutive_uptime += adjusted_uptime as u64;
        
        if self.uptime_stats.consecutive_uptime > self.uptime_stats.best_uptime_streak {
            self.uptime_stats.best_uptime_streak = self.uptime_stats.consecutive_uptime;
        }

        // Update bandwidth metrics with real network measurements
        self.update_bandwidth_utilization_with_network_data(&work, &bandwidth_stats).await?;

        // Update coverage metrics based on mesh topology
        self.update_coverage_metrics(&mesh_status, peer_count as usize).await?;

        // Update authenticity score with network validation
        self.update_authenticity_score_with_network_validation(&work, &mesh_status).await?;

        info!(
            "📊 ISP bypass work recorded with network validation: {}GB bandwidth, {}MB routed, {} users served, {:.1}% validated quality, {} peers",
            work.bandwidth_shared_gb, work.packets_routed_mb, work.users_served, validated_quality * 100.0, peer_count
        );

        Ok(())
    }

    /// Calculate comprehensive ISP bypass rewards using real consensus and blockchain data
    pub async fn calculate_rewards(&mut self, economic_model: &EconomicModel, network_stats: &NetworkStats) -> Result<TokenReward> {
        // Get real blockchain and consensus data
        let blockchain_health = get_blockchain_health().map_err(|e| anyhow::anyhow!("Blockchain health error: {}", e))?;
        let current_height = get_current_block_height().await.map_err(|e| anyhow::anyhow!("Block height error: {}", e))?;
        let mesh_status = get_mesh_status().await?;

        // Initialize consensus reward calculator for infrastructure validation
        let mut reward_calculator = RewardCalculator::new();

        // Base ISP bypass rewards calculated with network context
        let mut base_reward = TokenReward::calculate_isp_bypass(&self.current_work)?;

        // Apply real network utilization multipliers
        let network_utilization = self.calculate_real_network_utilization(&mesh_status).await?;
        let utilization_multiplier = if network_utilization > 0.8 {
            1.5 // High demand bonus
        } else if network_utilization > 0.5 {
            1.2 // Medium demand bonus
        } else {
            1.0 // Normal rate
        };

        // Calculate infrastructure rewards with blockchain validation
        let infrastructure_rewards = InfrastructureRewards::calculate_isp_bypass(&self.current_work)?;
        
        // Calculate quality bonuses based on real network performance
        let real_network_quality = self.calculate_network_quality_score(&mesh_status).await?;
        let quality_bonus = QualityBonus::calculate_isp_bypass_quality(
            real_network_quality,
            self.current_work.uptime_hours,
            base_reward.total_reward,
        )?;

        // Calculate network participation rewards with real peer data
        let peer_count = get_active_peer_count().await?;
        let participation_rewards = NetworkParticipationRewards::calculate(
            &self.current_work,
            peer_count as u32,
        )?;

        // Apply blockchain-validated reliability bonuses
        let reliability_multiplier = self.calculate_blockchain_validated_reliability(&blockchain_health).await?;
        
        // Apply network-validated authenticity score
        let authenticity_multiplier = self.authenticity_score;

        // Combine all reward sources with real network multipliers
        let total_base = base_reward.total_reward + 
                        infrastructure_rewards.total_infrastructure_rewards +
                        quality_bonus.total_bonus +
                        participation_rewards.total_participation_rewards;

        let network_adjusted_reward = ((total_base as f64) * utilization_multiplier * reliability_multiplier * authenticity_multiplier) as u64;

        // Apply final network consensus adjustments
        let consensus_adjustment = self.calculate_consensus_adjustment(current_height).await?;
        let final_reward = (network_adjusted_reward as f64 * consensus_adjustment) as u64;

        // Create comprehensive reward breakdown with real network data
        let comprehensive_reward = TokenReward {
            routing_reward: base_reward.routing_reward + participation_rewards.mesh_networking_rewards,
            storage_reward: 0, // ISP bypass doesn't include storage
            compute_reward: 0, // ISP bypass doesn't include compute
            quality_bonus: quality_bonus.total_bonus,
            uptime_bonus: base_reward.uptime_bonus + participation_rewards.connectivity_provision_rewards,
            total_reward: final_reward,
            currency: "ZHTP".to_string(),
        };

        // Record performance for historical analysis with blockchain timestamp
        self.record_performance_with_blockchain_data(&comprehensive_reward, current_height).await?;

        info!(
            "💰 ISP bypass rewards calculated with real network data: {} ZHTP (base: {}, network_util: {:.2}x, reliability: {:.2}x, authenticity: {:.2}x, consensus: {:.2}x)",
            comprehensive_reward.total_reward,
            total_base,
            utilization_multiplier,
            reliability_multiplier,
            authenticity_multiplier,
            consensus_adjustment
        );

        Ok(comprehensive_reward)
    }

    // Private helper methods using real network integrations

    /// Validate reported work against real network measurements
    async fn validate_work_against_network(
        &self, 
        work: &IspBypassWork, 
        mesh_status: &MeshStatus,
        bandwidth_stats: &BandwidthStatistics
    ) -> Result<()> {
        // Validate bandwidth claims against actual network interface statistics
        let max_theoretical_bandwidth = bandwidth_stats.upload_utilization * 24.0; // 24 hours max
        if work.bandwidth_shared_gb as f64 > max_theoretical_bandwidth * 2.0 {
            return Err(anyhow::anyhow!("Reported bandwidth exceeds network capacity by >200%"));
        }

        // Validate connection quality against mesh status
        if work.connection_quality > mesh_status.connectivity_percentage / 100.0 + 0.1 {
            return Err(anyhow::anyhow!("Reported quality exceeds network measurements"));
        }

        // Validate user count against peer connectivity
        let max_users_per_node = (mesh_status.active_peers / 2).max(1); // Conservative estimate
        if work.users_served > max_users_per_node as u64 {
            return Err(anyhow::anyhow!("Reported users served exceeds network topology limits"));
        }

        Ok(())
    }

    /// Calculate validated quality using real network measurements
    async fn calculate_validated_quality(
        &self,
        work: &IspBypassWork,
        mesh_status: &MeshStatus
    ) -> Result<f64> {
        // Blend reported quality with actual network measurements
        let network_quality = mesh_status.connectivity_percentage / 100.0;
        let reported_quality = work.connection_quality;
        
        // Weight network measurements higher for validation
        let validated_quality = (network_quality * 0.7 + reported_quality * 0.3).min(1.0).max(0.0);
        
        Ok(validated_quality)
    }

    /// Update bandwidth utilization with real network interface data
    async fn update_bandwidth_utilization_with_network_data(
        &mut self,
        work: &IspBypassWork,
        bandwidth_stats: &BandwidthStatistics
    ) -> Result<()> {
        // Use real network measurements for bandwidth metrics
        self.bandwidth_metrics.total_bandwidth_gb += work.bandwidth_shared_gb;
        self.bandwidth_metrics.avg_utilization = (bandwidth_stats.upload_utilization + bandwidth_stats.download_utilization) / 2.0;
        self.bandwidth_metrics.efficiency_score = bandwidth_stats.efficiency;
        
        // Calculate QoS based on actual network congestion
        let qos_score = match bandwidth_stats.congestion_level {
            CongestionLevel::Low => 0.9,
            CongestionLevel::Medium => 0.7,
            CongestionLevel::High => 0.5,
            CongestionLevel::Critical => 0.3,
        };
        self.bandwidth_metrics.qos_score = (self.bandwidth_metrics.qos_score + qos_score) / 2.0;

        Ok(())
    }

    /// Update coverage metrics based on real mesh topology
    async fn update_coverage_metrics(
        &mut self,
        mesh_status: &MeshStatus,
        peer_count: usize
    ) -> Result<()> {
        // Calculate coverage based on real mesh topology
        self.coverage_metrics.coverage_area_km2 = mesh_status.coverage * 100.0; // Convert to km²
        
        // Estimate population density based on peer distribution
        self.coverage_metrics.population_density = peer_count as f64 / self.coverage_metrics.coverage_area_km2.max(1.0);
        
        // Calculate rural/urban ratio based on mesh redundancy
        self.coverage_metrics.rural_urban_ratio = if mesh_status.redundancy > 0.7 {
            0.3 // High redundancy suggests urban
        } else {
            0.7 // Low redundancy suggests rural
        };
        
        // Calculate connectivity gap filled using mesh stability
        self.coverage_metrics.connectivity_gap_filled = mesh_status.stability * 0.8;

        Ok(())
    }

    /// Update authenticity score with network validation
    async fn update_authenticity_score_with_network_validation(
        &mut self,
        work: &IspBypassWork,
        mesh_status: &MeshStatus
    ) -> Result<()> {
        let mut authenticity_factors = Vec::new();

        // Factor 1: Network topology consistency
        let topology_consistency = if mesh_status.stability > 0.8 {
            1.0 // Stable network suggests authentic nodes
        } else if mesh_status.stability > 0.5 {
            0.8 // Moderate stability
        } else {
            0.6 // Unstable network suggests possible gaming
        };
        authenticity_factors.push(topology_consistency);

        // Factor 2: Mesh participation authenticity
        let mesh_authenticity = if mesh_status.mesh_connectivity && mesh_status.redundancy > 0.3 {
            1.0 // Good mesh participation
        } else {
            0.7 // Poor mesh participation
        };
        authenticity_factors.push(mesh_authenticity);

        // Factor 3: Bandwidth/user ratio validation against network capacity
        if work.users_served > 0 && work.bandwidth_shared_gb > 0 {
            let bandwidth_per_user = work.bandwidth_shared_gb as f64 / work.users_served as f64;
            let network_capacity_per_user = mesh_status.connectivity_percentage / mesh_status.active_peers as f64 * 10.0; // Estimate
            
            let ratio_authenticity = if bandwidth_per_user <= network_capacity_per_user * 1.5 {
                1.0 // Realistic ratio
            } else if bandwidth_per_user <= network_capacity_per_user * 3.0 {
                0.8 // High but possible
            } else {
                0.5 // Suspicious
            };
            authenticity_factors.push(ratio_authenticity);
        }

        // Calculate weighted authenticity score
        if !authenticity_factors.is_empty() {
            let new_score = authenticity_factors.iter().sum::<f64>() / authenticity_factors.len() as f64;
            self.authenticity_score = (self.authenticity_score * 0.85 + new_score * 0.15).max(0.1).min(1.0);
        }

        Ok(())
    }

    /// Calculate real network utilization from mesh statistics
    async fn calculate_real_network_utilization(
        &self,
        mesh_status: &MeshStatus
    ) -> Result<f64> {
        // Calculate utilization based on actual mesh metrics
        let connectivity_utilization = mesh_status.connectivity_percentage / 100.0;
        let peer_utilization = if mesh_status.active_peers > 0 {
            (mesh_status.active_peers as f64 / 50.0).min(1.0) // Normalize against target of 50 peers
        } else {
            0.0
        };
        
        // Weight different factors
        let overall_utilization = (connectivity_utilization * 0.6 + peer_utilization * 0.4).min(1.0);
        
        Ok(overall_utilization)
    }

    /// Calculate network quality score from real mesh measurements
    async fn calculate_network_quality_score(
        &self,
        mesh_status: &MeshStatus
    ) -> Result<f64> {
        // Combine multiple quality factors from real network data
        let connectivity_quality = mesh_status.connectivity_percentage / 100.0;
        let stability_quality = mesh_status.stability;
        let redundancy_quality = mesh_status.redundancy;
        
        // Weighted quality score
        let overall_quality = (connectivity_quality * 0.5 + stability_quality * 0.3 + redundancy_quality * 0.2).min(1.0);
        
        Ok(overall_quality)
    }

    /// Calculate blockchain-validated reliability multiplier
    async fn calculate_blockchain_validated_reliability(
        &self,
        blockchain_health: &lib_blockchain::BlockchainHealth
    ) -> Result<f64> {
        // Base reliability from uptime
        let uptime_reliability = if self.uptime_stats.total_uptime_hours > 0 {
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let total_time = (current_time - self.uptime_stats.session_start) / 3600;
            if total_time > 0 {
                (self.uptime_stats.total_uptime_hours as f64 / total_time as f64).min(1.0)
            } else {
                1.0
            }
        } else {
            0.5
        };

        // Blockchain sync bonus
        let blockchain_sync_bonus = if blockchain_health.is_synced {
            1.1 // 10% bonus for staying synced
        } else {
            0.9 // 10% penalty for being out of sync
        };

        // Network participation bonus based on blockchain connectivity
        let network_participation_bonus = if blockchain_health.peer_count > 5 {
            1.05 // 5% bonus for good peer connectivity
        } else {
            1.0
        };

        let total_reliability = uptime_reliability * blockchain_sync_bonus * network_participation_bonus;
        
        Ok(total_reliability.min(1.5).max(0.1)) // Cap between 10% and 150%
    }

    /// Calculate consensus-based reward adjustment
    async fn calculate_consensus_adjustment(&self, current_height: u64) -> Result<f64> {
        // Adjust rewards based on blockchain consensus state
        let height_factor = if current_height > 0 {
            // Small bonus for contributing to an active blockchain
            1.02
        } else {
            // No adjustment if blockchain isn't active
            1.0
        };

        // Future: could include validator consensus participation, network health, etc.
        Ok(height_factor)
    }

    /// Record performance with blockchain timestamp and validation
    async fn record_performance_with_blockchain_data(
        &mut self,
        reward: &TokenReward,
        block_height: u64
    ) -> Result<()> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let uptime_percentage = if self.uptime_stats.total_uptime_hours == 0 {
            0.0
        } else {
            let session_duration = (current_time - self.uptime_stats.session_start) / 3600;
            if session_duration > 0 {
                (self.uptime_stats.total_uptime_hours as f64 / session_duration as f64 * 100.0).min(100.0)
            } else {
                100.0
            }
        };

        let performance_record = IspBypassPerformanceRecord {
            timestamp: current_time,
            work_performed: self.current_work.clone(),
            reward_earned: reward.total_reward,
            quality_score: self.current_work.connection_quality,
            uptime_percentage,
            unique_users_served: self.current_work.users_served as u32,
        };

        self.performance_history.push(performance_record);

        // Keep only last 1000 records
        if self.performance_history.len() > 1000 {
            self.performance_history.remove(0);
        }

        info!(
            "📈 Performance recorded at block height {}: {} ZHTP reward for {:.1}% uptime",
            block_height, reward.total_reward, uptime_percentage
        );

        Ok(())
    }

    /// Distribute rewards to ISP bypass providers
    pub fn distribute_rewards(
        &self,
        participants: &mut [(&mut WalletBalance, &IspBypassWork)],
        total_reward_pool: u64,
    ) -> Result<()> {
        let mut distribution = RewardDistribution::new();
        distribution.distribute_isp_bypass_rewards(participants, total_reward_pool)?;
        
        info!(
            "🎯 Distributed {} ZHTP in ISP bypass rewards to {} participants",
            total_reward_pool, participants.len()
        );

        Ok(())
    }

    /// Get comprehensive ISP bypass statistics
    pub fn get_statistics(&self) -> serde_json::Value {
        let avg_quality = if self.quality_history.is_empty() {
            0.0
        } else {
            self.quality_history.iter().sum::<f64>() / self.quality_history.len() as f64
        };

        let uptime_percentage = if self.uptime_stats.total_uptime_hours == 0 {
            0.0
        } else {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let total_time = (current_time - self.uptime_stats.session_start) / 3600;
            if total_time > 0 {
                (self.uptime_stats.total_uptime_hours as f64 / total_time as f64 * 100.0).min(100.0)
            } else {
                100.0
            }
        };

        serde_json::json!({
            "current_work": {
                "bandwidth_shared_gb": self.current_work.bandwidth_shared_gb,
                "packets_routed_mb": self.current_work.packets_routed_mb,
                "users_served": self.current_work.users_served,
                "cost_savings_provided": self.current_work.cost_savings_provided,
                "connection_quality": self.current_work.connection_quality,
                "uptime_hours": self.current_work.uptime_hours
            },
            "historical_performance": {
                "total_cost_savings": self.total_cost_savings,
                "users_served_total": self.users_served_total,
                "avg_quality_score": avg_quality,
                "uptime_percentage": uptime_percentage,
                "performance_records": self.performance_history.len(),
                "best_uptime_streak": self.uptime_stats.best_uptime_streak,
                "downtime_incidents": self.uptime_stats.downtime_incidents
            },
            "bandwidth_metrics": {
                "total_bandwidth_gb": self.bandwidth_metrics.total_bandwidth_gb,
                "peak_capacity_mbps": self.bandwidth_metrics.peak_capacity_mbps,
                "avg_utilization": self.bandwidth_metrics.avg_utilization,
                "efficiency_score": self.bandwidth_metrics.efficiency_score,
                "qos_score": self.bandwidth_metrics.qos_score
            },
            "coverage_metrics": {
                "coverage_area_km2": self.coverage_metrics.coverage_area_km2,
                "population_density": self.coverage_metrics.population_density,
                "rural_urban_ratio": self.coverage_metrics.rural_urban_ratio,
                "connectivity_gap_filled": self.coverage_metrics.connectivity_gap_filled
            },
            "authenticity_score": self.authenticity_score,
            "reliability_multiplier": self.calculate_reliability_multiplier()
        })
    }

    /// Reset current work period (called after reward calculation)
    pub fn reset_work_period(&mut self) -> Result<()> {
        self.current_work = IspBypassWork::new();
        
        info!("🔄 ISP bypass work period reset for next calculation cycle");
        Ok(())
    }

    // Private helper methods

    fn update_bandwidth_utilization(&mut self, work: &IspBypassWork) {
        // Update utilization metrics based on current work
        if work.bandwidth_shared_gb > 0 {
            let current_utilization = (work.packets_routed_mb as f64 * 8.0) / (work.bandwidth_shared_gb as f64 * 1024.0); // Convert to utilization ratio
            self.bandwidth_metrics.avg_utilization = (self.bandwidth_metrics.avg_utilization + current_utilization) / 2.0;
        }

        // Update efficiency based on quality score
        self.bandwidth_metrics.efficiency_score = (self.bandwidth_metrics.efficiency_score + work.connection_quality) / 2.0;
        
        // Update QoS score based on user feedback (simulated)
        let qos_improvement = if work.users_served > 0 { work.connection_quality * 0.8 } else { 0.0 };
        self.bandwidth_metrics.qos_score = (self.bandwidth_metrics.qos_score + qos_improvement) / 2.0;
    }

    fn update_authenticity_score(&mut self, work: &IspBypassWork) {
        // Anti-Sybil detection based on work patterns
        let mut authenticity_factors = Vec::new();

        // Factor 1: Consistent quality over time
        if !self.quality_history.is_empty() {
            let quality_variance = self.calculate_quality_variance();
            let consistency_score = 1.0 - (quality_variance * 2.0).min(1.0); // Lower variance = higher authenticity
            authenticity_factors.push(consistency_score);
        }

        // Factor 2: Realistic bandwidth/user ratios
        if work.users_served > 0 && work.bandwidth_shared_gb > 0 {
            let bandwidth_per_user = work.bandwidth_shared_gb as f64 / work.users_served as f64;
            let realistic_ratio = if bandwidth_per_user > 0.1 && bandwidth_per_user < 100.0 {
                1.0 // Realistic ratio
            } else if bandwidth_per_user >= 100.0 {
                0.8 // Very high but possible
            } else {
                0.6 // Suspicious low bandwidth per user
            };
            authenticity_factors.push(realistic_ratio);
        }

        // Factor 3: Cost savings alignment with work
        if work.cost_savings_provided > 0 && work.bandwidth_shared_gb > 0 {
            let cost_per_gb = work.cost_savings_provided as f64 / work.bandwidth_shared_gb as f64;
            let realistic_cost = if cost_per_gb > 0.5 && cost_per_gb < 50.0 {
                1.0 // Realistic cost savings
            } else {
                0.7 // Questionable cost savings
            };
            authenticity_factors.push(realistic_cost);
        }

        // Calculate weighted authenticity score
        if !authenticity_factors.is_empty() {
            let new_score = authenticity_factors.iter().sum::<f64>() / authenticity_factors.len() as f64;
            self.authenticity_score = (self.authenticity_score * 0.9 + new_score * 0.1).max(0.1).min(1.0);
        }
    }

    fn calculate_quality_variance(&self) -> f64 {
        if self.quality_history.len() < 2 {
            return 0.0;
        }

        let mean = self.quality_history.iter().sum::<f64>() / self.quality_history.len() as f64;
        let variance = self.quality_history.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.quality_history.len() as f64;
        
        variance.sqrt() // Return standard deviation
    }

    fn calculate_reliability_multiplier(&self) -> f64 {
        let uptime_factor = if self.uptime_stats.total_uptime_hours > 0 {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let total_time = (current_time - self.uptime_stats.session_start) / 3600;
            if total_time > 0 {
                (self.uptime_stats.total_uptime_hours as f64 / total_time as f64).min(1.0)
            } else {
                1.0
            }
        } else {
            0.5
        };

        // Bonus for consistent uptime
        let consistency_bonus = if self.uptime_stats.downtime_incidents == 0 && uptime_factor > 0.95 {
            1.2 // 20% bonus for excellent reliability
        } else if uptime_factor > 0.9 {
            1.1 // 10% bonus for good reliability
        } else {
            1.0 // No bonus
        };

        uptime_factor * consistency_bonus
    }

    fn estimate_peer_connections(&self) -> u32 {
        // Estimate peer connections based on work patterns
        // In a real implementation, this would come from network layer
        let base_peers = (self.current_work.users_served / 3).max(1); // Assume each peer serves ~3 users
        let bandwidth_peers = (self.current_work.bandwidth_shared_gb / 10).max(1); // Assume 10GB per peer connection
        
        (base_peers + bandwidth_peers).min(50) as u32 // Cap at reasonable maximum
    }

    fn record_performance(&mut self, reward: &TokenReward) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let uptime_percentage = if self.uptime_stats.total_uptime_hours == 0 {
            0.0
        } else {
            let session_duration = (current_time - self.uptime_stats.session_start) / 3600;
            if session_duration > 0 {
                (self.uptime_stats.total_uptime_hours as f64 / session_duration as f64 * 100.0).min(100.0)
            } else {
                100.0
            }
        };

        let performance_record = IspBypassPerformanceRecord {
            timestamp: current_time,
            work_performed: self.current_work.clone(),
            reward_earned: reward.total_reward,
            quality_score: self.current_work.connection_quality,
            uptime_percentage,
            unique_users_served: self.current_work.users_served as u32,
        };

        self.performance_history.push(performance_record);

        // Keep only last 1000 records
        if self.performance_history.len() > 1000 {
            self.performance_history.remove(0);
        }
    }
}

impl Default for IspBypassRewardManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new ISP bypass reward manager
pub fn create_isp_bypass_reward_manager() -> IspBypassRewardManager {
    IspBypassRewardManager::new()
}

/// Calculate ISP bypass rewards for a given work period
pub async fn calculate_isp_bypass_rewards(
    work: &IspBypassWork,
    economic_model: &EconomicModel,
    network_stats: &NetworkStats,
) -> Result<TokenReward> {
    let mut manager = IspBypassRewardManager::new();
    manager.record_work(work.clone()).await?;
    manager.calculate_rewards(economic_model, network_stats).await
}

/// Batch process ISP bypass rewards for multiple providers
pub async fn batch_process_isp_bypass_rewards(
    providers: &mut HashMap<[u8; 32], (&mut WalletBalance, IspBypassWork)>,
    economic_model: &EconomicModel,
    network_stats: &NetworkStats,
) -> Result<u64> {
    let mut total_rewards = 0u64;

    for (node_id, (wallet, work)) in providers.iter_mut() {
        let reward = calculate_isp_bypass_rewards(work, economic_model, network_stats).await?;
        wallet.add_reward(&reward)?;
        total_rewards += reward.total_reward;

        info!(
            "💰 ISP bypass rewards processed for node {}: {} ZHTP",
            hex::encode(node_id),
            reward.total_reward
        );
    }

    info!(
        "🎯 Batch processed ISP bypass rewards: {} ZHTP total to {} providers",
        total_rewards, providers.len()
    );

    Ok(total_rewards)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EconomicModel;
    use crate::types::NetworkStats;

    #[test]
    fn test_isp_bypass_reward_manager_creation() {
        let manager = IspBypassRewardManager::new();
        
        assert_eq!(manager.current_work.bandwidth_shared_gb, 0);
        assert_eq!(manager.total_cost_savings, 0);
        assert_eq!(manager.users_served_total, 0);
        assert_eq!(manager.authenticity_score, 1.0);
        assert_eq!(manager.performance_history.len(), 0);
    }

    #[tokio::test]
    async fn test_work_recording() {
        let mut manager = IspBypassRewardManager::new();
        
        let work = IspBypassWork {
            bandwidth_shared_gb: 5, // Realistic value within 10 GB/hour capacity
            packets_routed_mb: 250, // Proportional to bandwidth
            uptime_hours: 12,
            connection_quality: 0.95,
            users_served: 3, // Reasonable number within 25 peer limit
            cost_savings_provided: 200, // Proportional cost savings
        };

        manager.record_work(work).await.unwrap();

        assert_eq!(manager.current_work.bandwidth_shared_gb, 5);
        assert_eq!(manager.current_work.packets_routed_mb, 250);
        assert_eq!(manager.total_cost_savings, 200);
        assert_eq!(manager.users_served_total, 3);
        assert_eq!(manager.quality_history.len(), 1);
        // Quality is validated using network measurements: (0.85 * 0.7 + 0.95 * 0.3) = 0.88
        assert!((manager.quality_history[0] - 0.88).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_reward_calculation() {
        let mut manager = IspBypassRewardManager::new();
        let economic_model = EconomicModel::new();
        let network_stats = NetworkStats::new();

        let work = IspBypassWork {
            bandwidth_shared_gb: 4, // Realistic value within capacity 
            packets_routed_mb: 200, // Proportional to bandwidth
            uptime_hours: 24,
            connection_quality: 0.9,
            users_served: 2, // Reasonable number 
            cost_savings_provided: 150, // Proportional cost savings
        };

        manager.record_work(work).await.unwrap();
        let reward = manager.calculate_rewards(&economic_model, &network_stats).await.unwrap();

        assert!(reward.total_reward > 0);
        assert!(reward.routing_reward > 0);
        assert_eq!(reward.storage_reward, 0); // ISP bypass doesn't include storage
        assert_eq!(reward.compute_reward, 0); // ISP bypass doesn't include compute
        assert_eq!(reward.currency, "ZHTP");
    }

    #[tokio::test]
    async fn test_authenticity_score_updates() {
        let mut manager = IspBypassRewardManager::new();

        // Record consistent, realistic work
        for _ in 0..5 {
            let work = IspBypassWork {
                bandwidth_shared_gb: 4, // Realistic value within capacity
                packets_routed_mb: 200, // Proportional to bandwidth
                uptime_hours: 12,
                connection_quality: 0.9,
                users_served: 3, // Reasonable number within peer limits
                cost_savings_provided: 150, // Proportional cost savings
            };
            manager.record_work(work).await.unwrap();
        }

        // Authenticity score should remain high for consistent work
        assert!(manager.authenticity_score > 0.8);

        // Record suspicious work pattern
        let suspicious_work = IspBypassWork {
            bandwidth_shared_gb: 15, // Slightly above capacity (10 GB/hour) but not extreme
            packets_routed_mb: 10,     // Very low routing for bandwidth
            uptime_hours: 12,
            connection_quality: 0.9,   // Realistic quality within network limits (0.85 + 0.1 = 0.95)
            users_served: 1,           // Very low users for high bandwidth
            cost_savings_provided: 10000, // Unrealistic cost savings
        };
        manager.record_work(suspicious_work).await.unwrap();

        // Authenticity score should decrease due to suspicious patterns
        assert!(manager.authenticity_score < 1.0);
    }

    #[test]
    fn test_reliability_multiplier() {
        let mut manager = IspBypassRewardManager::new();

        // Perfect uptime should give maximum multiplier
        manager.uptime_stats.total_uptime_hours = 100;
        manager.uptime_stats.downtime_incidents = 0;
        
        let multiplier = manager.calculate_reliability_multiplier();
        assert!(multiplier >= 1.0); // Should be at least 1.0 for good reliability
    }

    #[tokio::test]
    async fn test_statistics_generation() {
        let mut manager = IspBypassRewardManager::new();
        
        let work = IspBypassWork {
            bandwidth_shared_gb: 8, // Higher but realistic value within 10 GB/hour capacity 
            packets_routed_mb: 400, // Proportional to bandwidth
            uptime_hours: 48,
            connection_quality: 0.95,
            users_served: 4, // Reasonable number within peer limits
            cost_savings_provided: 400, // Proportional cost savings
        };

        manager.record_work(work).await.unwrap();
        let stats = manager.get_statistics();

        assert_eq!(stats["current_work"]["bandwidth_shared_gb"], 8);
        assert_eq!(stats["current_work"]["users_served"], 4);
        assert_eq!(stats["historical_performance"]["total_cost_savings"], 400);
        assert!(stats["authenticity_score"].as_f64().unwrap() > 0.0);
    }
}
