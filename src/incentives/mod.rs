//! Economic incentive systems module
//! 
//! Manages ISP bypass incentives and infrastructure rewards.

pub mod isp_bypass;
pub mod infrastructure_rewards;
pub mod quality_bonuses;
pub mod network_participation;
pub mod cost_savings;

pub use isp_bypass::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_isp_bypass_incentives_creation() {
        let incentives = IspBypassIncentives::new();
        
        assert_eq!(incentives.connectivity_sharing_rate, crate::ISP_BYPASS_CONNECTIVITY_RATE);
        assert_eq!(incentives.mesh_routing_rate, crate::ISP_BYPASS_MESH_RATE);
        assert_eq!(incentives.uptime_bonus_rate, crate::ISP_BYPASS_UPTIME_BONUS);
        assert_eq!(incentives.bandwidth_quality_multiplier, 1.5);
        assert_eq!(incentives.total_bandwidth_shared, 0);
        assert_eq!(incentives.total_isp_cost_savings, 0);
        assert_eq!(incentives.connectivity_providers, 0);
        assert_eq!(incentives.mesh_participants, 0);
    }

    #[test]
    fn test_isp_bypass_reward_calculation() {
        let incentives = IspBypassIncentives::new();
        
        let work = IspBypassWork {
            bandwidth_shared_gb: 10,
            packets_routed_mb: 500,
            uptime_hours: 24,
            connection_quality: 0.95, // High quality
            users_served: 8,
            cost_savings_provided: 400,
        };
        
        let reward = incentives.calculate_rewards(&work);
        
        // Calculate expected reward
        let bandwidth_reward = 10 * incentives.connectivity_sharing_rate; // 10 * 100 = 1000
        let routing_reward = 500 * incentives.mesh_routing_rate; // 500 * 1 = 500
        let uptime_bonus = 24 * incentives.uptime_bonus_rate; // 24 * 10 = 240
        let base_reward = bandwidth_reward + routing_reward + uptime_bonus; // 1740
        let quality_multiplied = ((base_reward as f64) * 1.5) as u64; // 1740 * 1.5 = 2610
        
        assert_eq!(reward, quality_multiplied);
    }

    #[test]
    fn test_isp_bypass_reward_without_quality_bonus() {
        let incentives = IspBypassIncentives::new();
        
        let work = IspBypassWork {
            bandwidth_shared_gb: 5,
            packets_routed_mb: 200,
            uptime_hours: 12,
            connection_quality: 0.8, // Below quality threshold
            users_served: 3,
            cost_savings_provided: 150,
        };
        
        let reward = incentives.calculate_rewards(&work);
        
        // Calculate expected reward without quality bonus
        let bandwidth_reward = 5 * incentives.connectivity_sharing_rate; // 5 * 100 = 500
        let routing_reward = 200 * incentives.mesh_routing_rate; // 200 * 1 = 200
        let uptime_bonus = 12 * incentives.uptime_bonus_rate; // 12 * 10 = 120
        let base_reward = bandwidth_reward + routing_reward + uptime_bonus; // 820
        
        assert_eq!(reward, base_reward); // No quality multiplier applied
    }

    #[test]
    fn test_isp_bypass_stats_update() {
        let mut incentives = IspBypassIncentives::new();
        
        let work1 = IspBypassWork {
            bandwidth_shared_gb: 5,
            packets_routed_mb: 100,
            uptime_hours: 12,
            connection_quality: 0.9,
            users_served: 3,
            cost_savings_provided: 120,
        };
        
        let work2 = IspBypassWork {
            bandwidth_shared_gb: 8,
            packets_routed_mb: 150,
            uptime_hours: 18,
            connection_quality: 0.95,
            users_served: 5,
            cost_savings_provided: 200,
        };
        
        // Update stats with first work
        incentives.update_stats(&work1).unwrap();
        assert_eq!(incentives.total_bandwidth_shared, 5);
        assert_eq!(incentives.total_isp_cost_savings, 120);
        assert_eq!(incentives.connectivity_providers, 1);
        assert_eq!(incentives.mesh_participants, 1);
        assert_eq!(incentives.avg_cost_savings_per_user, 120);
        
        // Update stats with second work
        incentives.update_stats(&work2).unwrap();
        assert_eq!(incentives.total_bandwidth_shared, 13); // 5 + 8
        assert_eq!(incentives.total_isp_cost_savings, 320); // 120 + 200
        assert_eq!(incentives.connectivity_providers, 2);
        assert_eq!(incentives.mesh_participants, 2);
        assert_eq!(incentives.avg_cost_savings_per_user, 160); // 320 / 2
    }

    #[test]
    fn test_isp_bypass_stats_with_zero_bandwidth() {
        let mut incentives = IspBypassIncentives::new();
        
        let work = IspBypassWork {
            bandwidth_shared_gb: 0, // No bandwidth shared
            packets_routed_mb: 50,
            uptime_hours: 6,
            connection_quality: 0.8,
            users_served: 1,
            cost_savings_provided: 25,
        };
        
        incentives.update_stats(&work).unwrap();
        
        // Should not count as connectivity provider if no bandwidth shared
        assert_eq!(incentives.total_bandwidth_shared, 0);
        assert_eq!(incentives.connectivity_providers, 0); // No bandwidth = no provider
        assert_eq!(incentives.mesh_participants, 1); // But still counts as mesh participant
        assert_eq!(incentives.avg_cost_savings_per_user, 0); // Division by zero protection
    }

    #[test]
    fn test_isp_bypass_stats_json() {
        let incentives = IspBypassIncentives::new();
        let stats = incentives.get_stats();
        
        // Verify all required fields are present
        assert!(stats["total_bandwidth_shared_gb"].is_u64());
        assert!(stats["total_isp_cost_savings_usd"].is_u64());
        assert!(stats["connectivity_providers"].is_u64());
        assert!(stats["mesh_participants"].is_u64());
        assert!(stats["avg_cost_savings_per_user_usd"].is_u64());
        assert!(stats["connectivity_sharing_rate"].is_u64());
        assert!(stats["mesh_routing_rate"].is_u64());
        assert!(stats["uptime_bonus_rate"].is_u64());
        
        // Check initial values
        assert_eq!(stats["total_bandwidth_shared_gb"], 0);
        assert_eq!(stats["connectivity_sharing_rate"], crate::ISP_BYPASS_CONNECTIVITY_RATE);
    }

    #[test]
    fn test_work_metrics_isp_bypass_value() {
        let mut work = IspBypassWork::new();
        
        // Test initial state
        assert_eq!(work.total_isp_bypass_value(), 0);
        
        // Add some work
        work.add_bandwidth_shared(5);
        work.add_packets_routed(100);
        work.uptime_hours = 12;
        work.update_connection_quality(0.95);
        
        let value = work.total_isp_bypass_value();
        
        // Expected calculation
        let bandwidth_reward = 5 * crate::ISP_BYPASS_CONNECTIVITY_RATE; // 5 * 100 = 500
        let routing_reward = 100 * crate::ISP_BYPASS_MESH_RATE; // 100 * 1 = 100
        let uptime_bonus = 12 * crate::ISP_BYPASS_UPTIME_BONUS; // 12 * 10 = 120
        let base_reward = bandwidth_reward + routing_reward + uptime_bonus; // 720
        let quality_multiplied = ((base_reward as f64) * 1.5) as u64; // 720 * 1.5 = 1080
        
        assert_eq!(value, quality_multiplied);
    }

    #[test]
    fn test_work_metrics_helpers() {
        let mut work = IspBypassWork::new();
        
        // Test helper methods
        work.add_bandwidth_shared(3);
        work.add_packets_routed(75);
        work.add_users_served(2);
        work.add_cost_savings(180);
        work.update_connection_quality(0.88);
        
        assert_eq!(work.bandwidth_shared_gb, 3);
        assert_eq!(work.packets_routed_mb, 75);
        assert_eq!(work.users_served, 2);
        assert_eq!(work.cost_savings_provided, 180);
        assert_eq!(work.connection_quality, 0.88);
    }

    #[test]
    fn test_work_metrics_quality_bounds() {
        let mut work = IspBypassWork::new();
        
        // Test quality score bounds
        work.update_connection_quality(1.5); // Above 1.0
        assert_eq!(work.connection_quality, 1.0); // Should be clamped to 1.0
        
        work.update_connection_quality(-0.5); // Below 0.0
        assert_eq!(work.connection_quality, 0.0); // Should be clamped to 0.0
        
        work.update_connection_quality(0.75); // Normal value
        assert_eq!(work.connection_quality, 0.75); // Should remain unchanged
    }
}
