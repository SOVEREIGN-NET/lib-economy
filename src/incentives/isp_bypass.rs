//! ISP Bypass Economics - incentives for replacing traditional ISPs
//! 
//! Manages economic incentives for Internet Service Provider replacement.

use anyhow::Result;
use serde::{Serialize, Deserialize};
use crate::types::IspBypassWork;
use crate::wasm::logging::info;

/// ISP Bypass Economics - incentives for replacing traditional ISPs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IspBypassIncentives {
    /// Reward for sharing internet connectivity (ZHTP per GB shared)
    pub connectivity_sharing_rate: u64,
    /// Reward for routing packets through mesh (ZHTP per MB routed)
    pub mesh_routing_rate: u64,
    /// Bonus for maintaining 24/7 connectivity sharing
    pub uptime_bonus_rate: u64,
    /// Reward for providing high-bandwidth connections
    pub bandwidth_quality_multiplier: f64,
    /// Total internet bandwidth shared by network (GB/month)
    pub total_bandwidth_shared: u64,
    /// Total cost savings from ISP bypass (USD equivalent)
    pub total_isp_cost_savings: u64,
    /// Number of participants sharing connectivity
    pub connectivity_providers: u64,
    /// Number of participants in mesh network
    pub mesh_participants: u64,
    /// Average cost savings per participant (USD/month)
    pub avg_cost_savings_per_user: u64,
}

impl IspBypassIncentives {
    /// Create new ISP bypass incentive structure
    pub fn new() -> Self {
        IspBypassIncentives {
            connectivity_sharing_rate: crate::ISP_BYPASS_CONNECTIVITY_RATE, // 100 ZHTP per GB
            mesh_routing_rate: crate::ISP_BYPASS_MESH_RATE,                 // 1 ZHTP per MB
            uptime_bonus_rate: crate::ISP_BYPASS_UPTIME_BONUS,              // 10 ZHTP per hour
            bandwidth_quality_multiplier: 1.5,                             // 1.5x for high quality
            total_bandwidth_shared: 0,
            total_isp_cost_savings: 0,
            connectivity_providers: 0,
            mesh_participants: 0,
            avg_cost_savings_per_user: 50, // Estimated $50/month savings
        }
    }
    
    /// Update ISP bypass statistics
    pub fn update_stats(&mut self, new_work: &IspBypassWork) -> Result<()> {
        self.total_bandwidth_shared += new_work.bandwidth_shared_gb;
        self.total_isp_cost_savings += new_work.cost_savings_provided;
        
        if new_work.bandwidth_shared_gb > 0 {
            self.connectivity_providers += 1;
        }
        
        if new_work.packets_routed_mb > 0 {
            self.mesh_participants += 1;
        }
        
        // Update average cost savings
        if self.connectivity_providers > 0 {
            self.avg_cost_savings_per_user = self.total_isp_cost_savings / self.connectivity_providers;
        } else {
            self.avg_cost_savings_per_user = 0; // No providers = no cost savings
        }
        
        info!(
            "Updated ISP bypass stats: {} GB shared, {} providers, ${} total savings",
            self.total_bandwidth_shared, self.connectivity_providers, self.total_isp_cost_savings
        );
        
        Ok(())
    }
    
    /// Calculate ISP bypass rewards for work performed
    pub fn calculate_rewards(&self, work: &IspBypassWork) -> u64 {
        // Bandwidth sharing reward
        let bandwidth_reward = work.bandwidth_shared_gb * self.connectivity_sharing_rate;
        
        // Packet routing reward
        let routing_reward = work.packets_routed_mb * self.mesh_routing_rate;
        
        // Uptime bonus
        let uptime_bonus = work.uptime_hours * self.uptime_bonus_rate;
        
        // Quality multiplier
        let base_reward = bandwidth_reward + routing_reward + uptime_bonus;
        let quality_multiplied = if work.connection_quality > 0.9 {
            ((base_reward as f64) * self.bandwidth_quality_multiplier) as u64
        } else {
            base_reward
        };
        
        quality_multiplied
    }
    
    /// Get ISP bypass statistics
    pub fn get_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "total_bandwidth_shared_gb": self.total_bandwidth_shared,
            "total_isp_cost_savings_usd": self.total_isp_cost_savings,
            "connectivity_providers": self.connectivity_providers,
            "mesh_participants": self.mesh_participants,
            "avg_cost_savings_per_user_usd": self.avg_cost_savings_per_user,
            "connectivity_sharing_rate": self.connectivity_sharing_rate,
            "mesh_routing_rate": self.mesh_routing_rate,
            "uptime_bonus_rate": self.uptime_bonus_rate
        })
    }
}

impl Default for IspBypassIncentives {
    fn default() -> Self {
        Self::new()
    }
}
