//! Supply management implementation
use crate::types::*;

/// Manages token supply based on post-scarcity economics
pub struct SupplyManager {
    pub current_supply: u64,
    pub max_supply: u64,
}

impl SupplyManager {
    pub fn new() -> Self {
        Self {
            current_supply: 0,
            max_supply: u64::MAX, // Post-scarcity: unlimited supply
        }
    }

    /// Calculate new tokens to mint based on economic activity
    pub fn calculate_mint_amount(&self, infrastructure_usage: u64, network_activity: u64) -> u64 {
        // Post-scarcity model: mint based on utility, not scarcity
        let base_mint = infrastructure_usage * 10; // 10 ZHTP per unit of infrastructure
        let activity_bonus = network_activity * 5; // 5 ZHTP per unit of network activity
        
        base_mint + activity_bonus
    }

    /// Mint new tokens (utility-based, not speculation-based)
    pub fn mint_tokens(&mut self, amount: u64) -> Result<(), String> {
        if self.current_supply.checked_add(amount).is_none() {
            return Err("Supply overflow".to_string());
        }
        
        self.current_supply += amount;
        Ok(())
    }

    /// Get current supply statistics
    pub fn get_supply_stats(&self) -> (u64, u64, f64) {
        let utilization = if self.max_supply == u64::MAX {
            0.0 // Post-scarcity: never reaches capacity
        } else {
            self.current_supply as f64 / self.max_supply as f64
        };
        
        (self.current_supply, self.max_supply, utilization)
    }
}
