# Refactoring Plan: DAO Governance Rules

## Overview

This document outlines the architectural refactoring needed to move DAO ownership validation rules from `lib-identity` to `lib-economy` where they properly belong.

## Current State (As of November 2025)

### Implementation Location: `lib-identity` ❌

Currently, DAO ownership rules are implemented in the identity/wallet layer:

**File**: `lib-identity/src/wallets/manager_integration.rs`

**Functions with Business Rules**:
1. `establish_dao_hierarchy()` - Lines 747-753
2. `authorize_dao_controller()` - Lines 806-812

**Current Implementation**:
```rust
// In lib-identity/src/wallets/manager_integration.rs

pub fn establish_dao_hierarchy(
    &mut self,
    parent_dao_id: &WalletId,
    child_dao_id: &WalletId,
    authorized_by: IdentityId,
) -> Result<()> {
    // ... authorization checks ...
    
    // Business rule: Non-profit DAOs cannot own for-profit DAOs
    let parent_wallet_type = self.wallets.get(parent_dao_id).unwrap().wallet_type.clone();
    let child_wallet_type = self.wallets.get(child_dao_id).unwrap().wallet_type.clone();
    
    if parent_wallet_type == WalletType::NonProfitDAO && child_wallet_type == WalletType::ForProfitDAO {
        return Err(anyhow!(
            "Non-profit DAO cannot own or control a for-profit DAO. \
            This violates organizational governance rules."
        ));
    }
    
    // ... relationship establishment ...
}

pub fn authorize_dao_controller(
    &mut self,
    target_dao_id: &WalletId,
    controller_dao_id: &WalletId,
    authorized_by: IdentityId,
) -> Result<()> {
    // ... authorization checks ...
    
    // Business rule: Non-profit DAOs cannot control for-profit DAOs
    let controller_wallet_type = self.wallets.get(controller_dao_id).unwrap().wallet_type.clone();
    let target_wallet_type = self.wallets.get(target_dao_id).unwrap().wallet_type.clone();
    
    if controller_wallet_type == WalletType::NonProfitDAO && target_wallet_type == WalletType::ForProfitDAO {
        return Err(anyhow!(
            "Non-profit DAO cannot be authorized as controller of a for-profit DAO. \
            This violates organizational governance rules."
        ));
    }
    
    // ... controller authorization ...
}
```

**Test Coverage**: `lib-identity/src/wallets/dao_hierarchy_demo.rs`
- `test_nonprofit_cannot_own_forprofit()` - Validates rejection
- `test_forprofit_can_own_nonprofit()` - Validates approval

## Architectural Problems

### 1. **Violation of Separation of Concerns**
- `lib-identity` should handle: wallets, DIDs, cryptographic identities
- `lib-economy` should handle: economic rules, governance policies, DAO regulations
- **Current state**: Identity layer contains economic policy logic

### 2. **Tight Coupling**
- Identity/wallet infrastructure is coupled to economic governance rules
- Changing economic policy requires modifying identity layer
- Makes both modules harder to test and maintain independently

### 3. **Limited Extensibility**
- Adding new DAO types or ownership rules requires changes to identity code
- Economic simulations/modeling can't be done without identity infrastructure
- Violates Single Responsibility Principle

### 4. **Unclear Ownership**
- Developers looking for DAO governance rules would naturally check `lib-economy`
- Current location creates confusion about where economic policy lives

## Proposed Refactoring

### Target Architecture ✅

Move DAO ownership validation to `lib-economy` as a dedicated governance module:

```
lib-economy/
  src/
    dao_governance/          # NEW MODULE
      mod.rs
      ownership_rules.rs     # Ownership validation logic
      compliance.rs          # Compliance checking utilities
      policy.rs              # Policy definitions
```

### New Module: `lib-economy/src/dao_governance/ownership_rules.rs`

```rust
//! DAO Ownership and Control Rules
//! 
//! This module enforces organizational governance rules for DAO hierarchies
//! and controller relationships in the ZHTP economic system.

use anyhow::{Result, anyhow};

/// DAO organizational types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaoOrganizationType {
    /// Non-profit organization (no private ownership)
    NonProfit,
    /// For-profit organization (can be privately owned)
    ForProfit,
}

/// DAO ownership rule validator
pub struct DaoOwnershipValidator;

impl DaoOwnershipValidator {
    /// Validate parent-child DAO hierarchy relationship
    /// 
    /// # Business Rules
    /// - Non-profit DAOs **cannot** own or control for-profit DAOs
    /// - For-profit DAOs **can** own/control both non-profit and for-profit DAOs
    /// - This enforces organizational integrity and prevents conflicts of interest
    /// 
    /// # Returns
    /// - `Ok(())` if the relationship is valid
    /// - `Err(...)` with detailed explanation if invalid
    pub fn validate_hierarchy(
        parent_type: DaoOrganizationType,
        child_type: DaoOrganizationType,
    ) -> Result<()> {
        match (parent_type, child_type) {
            // Non-profit cannot own for-profit
            (DaoOrganizationType::NonProfit, DaoOrganizationType::ForProfit) => {
                Err(anyhow!(
                    "Non-profit DAO cannot own or control a for-profit DAO. \
                    This violates organizational governance rules and creates \
                    conflicts of interest in the post-scarcity economic model."
                ))
            }
            // All other combinations are valid
            _ => Ok(()),
        }
    }
    
    /// Validate controller authorization relationship
    /// 
    /// # Business Rules
    /// - Non-profit DAOs **cannot** be authorized as controllers of for-profit DAOs
    /// - For-profit DAOs **can** control both non-profit and for-profit DAOs
    /// 
    /// # Returns
    /// - `Ok(())` if the controller relationship is valid
    /// - `Err(...)` with detailed explanation if invalid
    pub fn validate_controller(
        controller_type: DaoOrganizationType,
        target_type: DaoOrganizationType,
    ) -> Result<()> {
        match (controller_type, target_type) {
            // Non-profit cannot control for-profit
            (DaoOrganizationType::NonProfit, DaoOrganizationType::ForProfit) => {
                Err(anyhow!(
                    "Non-profit DAO cannot be authorized as controller of a for-profit DAO. \
                    This violates organizational governance rules and economic policy integrity."
                ))
            }
            // All other combinations are valid
            _ => Ok(()),
        }
    }
    
    /// Get all valid child types for a given parent type
    pub fn get_valid_child_types(parent_type: DaoOrganizationType) -> Vec<DaoOrganizationType> {
        match parent_type {
            // Non-profit can only own non-profit
            DaoOrganizationType::NonProfit => vec![DaoOrganizationType::NonProfit],
            // For-profit can own both
            DaoOrganizationType::ForProfit => vec![
                DaoOrganizationType::NonProfit,
                DaoOrganizationType::ForProfit,
            ],
        }
    }
    
    /// Get all valid controller types for a given target type
    pub fn get_valid_controller_types(target_type: DaoOrganizationType) -> Vec<DaoOrganizationType> {
        match target_type {
            // For-profit can only be controlled by for-profit
            DaoOrganizationType::ForProfit => vec![DaoOrganizationType::ForProfit],
            // Non-profit can be controlled by both
            DaoOrganizationType::NonProfit => vec![
                DaoOrganizationType::NonProfit,
                DaoOrganizationType::ForProfit,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nonprofit_cannot_own_forprofit() {
        let result = DaoOwnershipValidator::validate_hierarchy(
            DaoOrganizationType::NonProfit,
            DaoOrganizationType::ForProfit,
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Non-profit DAO cannot own"));
    }
    
    #[test]
    fn test_forprofit_can_own_nonprofit() {
        let result = DaoOwnershipValidator::validate_hierarchy(
            DaoOrganizationType::ForProfit,
            DaoOrganizationType::NonProfit,
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_forprofit_can_own_forprofit() {
        let result = DaoOwnershipValidator::validate_hierarchy(
            DaoOrganizationType::ForProfit,
            DaoOrganizationType::ForProfit,
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_nonprofit_can_own_nonprofit() {
        let result = DaoOwnershipValidator::validate_hierarchy(
            DaoOrganizationType::NonProfit,
            DaoOrganizationType::NonProfit,
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_nonprofit_cannot_control_forprofit() {
        let result = DaoOwnershipValidator::validate_controller(
            DaoOrganizationType::NonProfit,
            DaoOrganizationType::ForProfit,
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be authorized as controller"));
    }
    
    #[test]
    fn test_valid_child_types() {
        let nonprofit_children = DaoOwnershipValidator::get_valid_child_types(
            DaoOrganizationType::NonProfit
        );
        assert_eq!(nonprofit_children.len(), 1);
        assert_eq!(nonprofit_children[0], DaoOrganizationType::NonProfit);
        
        let forprofit_children = DaoOwnershipValidator::get_valid_child_types(
            DaoOrganizationType::ForProfit
        );
        assert_eq!(forprofit_children.len(), 2);
    }
}
```

### Updated `lib-identity` Integration

After refactoring, `lib-identity` would call `lib-economy` for validation:

```rust
// In lib-identity/src/wallets/manager_integration.rs

use lib_economy::dao_governance::{DaoOwnershipValidator, DaoOrganizationType};

pub fn establish_dao_hierarchy(
    &mut self,
    parent_dao_id: &WalletId,
    child_dao_id: &WalletId,
    authorized_by: IdentityId,
) -> Result<()> {
    // ... authorization checks ...
    
    // Convert wallet types to organization types
    let parent_org_type = self.get_dao_organization_type(parent_dao_id)?;
    let child_org_type = self.get_dao_organization_type(child_dao_id)?;
    
    // Validate using lib-economy rules
    DaoOwnershipValidator::validate_hierarchy(parent_org_type, child_org_type)?;
    
    // ... relationship establishment ...
}

pub fn authorize_dao_controller(
    &mut self,
    target_dao_id: &WalletId,
    controller_dao_id: &WalletId,
    authorized_by: IdentityId,
) -> Result<()> {
    // ... authorization checks ...
    
    // Convert wallet types to organization types
    let controller_org_type = self.get_dao_organization_type(controller_dao_id)?;
    let target_org_type = self.get_dao_organization_type(target_dao_id)?;
    
    // Validate using lib-economy rules
    DaoOwnershipValidator::validate_controller(controller_org_type, target_org_type)?;
    
    // ... controller authorization ...
}

// Helper method to convert wallet type to organization type
fn get_dao_organization_type(&self, dao_id: &WalletId) -> Result<DaoOrganizationType> {
    let wallet = self.wallets.get(dao_id)
        .ok_or_else(|| anyhow!("DAO wallet not found"))?;
    
    match wallet.wallet_type {
        WalletType::NonProfitDAO => Ok(DaoOrganizationType::NonProfit),
        WalletType::ForProfitDAO => Ok(DaoOrganizationType::ForProfit),
        _ => Err(anyhow!("Wallet is not a DAO type")),
    }
}
```

## Refactoring Steps

### Phase 1: Create New Module in lib-economy
1. Create `lib-economy/src/dao_governance/` directory
2. Implement `ownership_rules.rs` with validation logic
3. Implement `mod.rs` to export public API
4. Add comprehensive tests
5. Update `lib-economy/src/lib.rs` to expose new module

### Phase 2: Update lib-identity Dependencies
1. Add `lib-economy` as dependency in `lib-identity/Cargo.toml`
2. Update `manager_integration.rs` to use new validation API
3. Add helper method `get_dao_organization_type()`
4. Remove inline business logic checks

### Phase 3: Testing & Validation
1. Run all existing tests in `lib-identity/src/wallets/dao_hierarchy_demo.rs`
2. Verify behavior is identical to current implementation
3. Add integration tests in `lib-economy/tests/dao_governance_tests.rs`
4. Ensure backward compatibility

### Phase 4: Documentation Updates
1. Update `lib-economy/docs/ARCHITECTURE.md` with DAO governance section
2. Add `lib-economy/docs/DAO_GOVERNANCE.md` with detailed rules
3. Update `lib-identity` docs to reference `lib-economy` for policy rules
4. Add migration notes for developers

## Benefits of Refactoring

### 1. **Proper Separation of Concerns**
- Identity layer focuses on wallets, DIDs, cryptographic identity
- Economic layer owns all economic policy and governance rules
- Clear boundaries between infrastructure and policy

### 2. **Improved Maintainability**
- Economic rules can be modified without touching identity code
- Easier to understand where to make policy changes
- Reduced coupling between modules

### 3. **Enhanced Extensibility**
- Easy to add new DAO types (e.g., Hybrid, Cooperative, etc.)
- Can create complex governance policies without identity changes
- Supports economic modeling and simulation tools

### 4. **Better Testability**
- Economic rules can be tested independently
- Easier to create test scenarios for different policies
- Cleaner test structure

### 5. **Clearer Documentation**
- Governance rules clearly documented in `lib-economy`
- Developers know where to look for economic policy
- API documentation more focused

## Future Extensions

Once refactored, this architecture enables:

### 1. **Policy Configuration**
```rust
// Future: Load governance rules from config
pub struct DaoGovernancePolicy {
    pub allow_nonprofit_owning_forprofit: bool,
    pub require_transparency_for_nonprofits: bool,
    pub max_hierarchy_depth: Option<usize>,
}
```

### 2. **Dynamic Rule Engine**
```rust
// Future: Pluggable rule system
pub trait DaoGovernanceRule {
    fn validate(&self, parent: &DaoInfo, child: &DaoInfo) -> Result<()>;
}
```

### 3. **Compliance Reporting**
```rust
// Future: Generate compliance reports
pub fn generate_dao_compliance_report(dao_id: &DaoId) -> ComplianceReport {
    // Check all governance rules, generate audit trail
}
```

### 4. **Cross-Chain Governance**
```rust
// Future: Validate cross-chain DAO relationships
pub fn validate_cross_chain_hierarchy(
    chain_a_dao: &DaoInfo,
    chain_b_dao: &DaoInfo,
) -> Result<()>;
```

## Migration Timeline

**Estimated Effort**: 2-3 days

- **Day 1**: Implement new `lib-economy` module with full test coverage
- **Day 2**: Refactor `lib-identity` to use new API, update tests
- **Day 3**: Documentation updates, integration testing, code review

## Backward Compatibility

This refactoring is **API-compatible** at the public interface level:
- `WalletManager::establish_dao_hierarchy()` signature unchanged
- `WalletManager::authorize_dao_controller()` signature unchanged
- Same error messages and behavior
- Existing tests pass without modification

## Conclusion

Moving DAO governance rules from `lib-identity` to `lib-economy` represents a significant architectural improvement that:
- Aligns with proper separation of concerns
- Improves maintainability and extensibility
- Creates clearer ownership of economic policy
- Enables future enhancements to governance systems

This refactoring should be prioritized as **technical debt reduction** and implemented before adding new DAO features or governance mechanisms.

---

**Status**: Documented for future implementation  
**Priority**: Medium (Technical Debt)  
**Complexity**: Low-Medium  
**Risk**: Low (Backward compatible)  
**Dependencies**: None (purely internal refactoring)
