use ethereum_types::U256;

use crate::model::{
    revision::Revision,
};

use crate::model::evmc::{
    AccessStatus, StorageStatus,
};

/// calculate gas cost
/// 
/// sstore on Constantinople: https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1283.md
/// 
/// sstore on Istanbul: https://github.com/sorpaas/EIPs/blob/sp-eip-new-net-metering/EIPS/eip-2200.md
/// 
/// sstore on Berlin: https://eips.ethereum.org/EIPS/eip-2929
pub fn calc_sstore_gas_cost(new_value: U256, revision: Revision, access_status: AccessStatus, status: StorageStatus) -> i64 {    
    let is_eip1283 = revision >= Revision::Istanbul || revision == Revision::Constantinople;
    
    if !is_eip1283 {
        if status.current.is_zero() && !new_value.is_zero() {
            return SSTORE_SET_GAS;
        }else{
            return SSTORE_RESET_GAS;
        }
    }
    
    // unchanged: current == new_value
    if status.current == new_value {
        return match (revision, access_status) {
            (Revision::Berlin, AccessStatus::Cold) | (Revision::London, AccessStatus::Cold) | (Revision::Shanghai, AccessStatus::Cold) => 2200,
            (Revision::Berlin, AccessStatus::Warm) | (Revision::London, AccessStatus::Warm) | (Revision::Shanghai, AccessStatus::Warm) => 100,
            (Revision::Istanbul, _) => 800,
            (Revision::Constantinople, _) => 200,
            _ => 5000
        };
    }
    
    if status.original == status.current {
        if status.original.is_zero() {
            20000 + match (revision, access_status) {
                (Revision::Berlin, AccessStatus::Cold) | (Revision::London, AccessStatus::Cold) | (Revision::Shanghai, AccessStatus::Cold) => 2100,
                (Revision::Berlin, AccessStatus::Warm) | (Revision::London, AccessStatus::Warm) | (Revision::Shanghai, AccessStatus::Warm) => 0,
                _ => 0,
            }
        }else{
            5000 + match (revision, access_status) {
                (Revision::Berlin, AccessStatus::Cold) | (Revision::London, AccessStatus::Cold) | (Revision::Shanghai, AccessStatus::Cold) => 0,
                (Revision::Berlin, AccessStatus::Warm) | (Revision::London, AccessStatus::Warm) | (Revision::Shanghai, AccessStatus::Warm) => -2100,
                _ => 0,
            }
        }
    }else{
        match (revision, access_status) {
            (Revision::Berlin, AccessStatus::Cold) | (Revision::London, AccessStatus::Cold) | (Revision::Shanghai, AccessStatus::Cold) => 2200,
            (Revision::Berlin, AccessStatus::Warm) | (Revision::London, AccessStatus::Warm) | (Revision::Shanghai, AccessStatus::Warm) => 100,
            (Revision::Istanbul, _) => 800,
            (Revision::Constantinople, _) => 200,
            _ => 5000,
        }
    }
}

/// calculate gas refund
/// 
/// sstore on Constantinople: https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1283.md
/// 
/// sstore on Istanbul: https://github.com/sorpaas/EIPs/blob/sp-eip-new-net-metering/EIPS/eip-2200.md
/// 
/// sstore on Berlin: https://eips.ethereum.org/EIPS/eip-2929
/// 
/// sstore on London: https://eips.ethereum.org/EIPS/eip-3529
pub fn calc_sstore_gas_refund(new_value: U256, revision: Revision, status: StorageStatus) -> i64 {
    let is_eip1283 = revision >= Revision::Istanbul || revision == Revision::Constantinople;
    
    if !is_eip1283 {
        if status.current != new_value && !status.current.is_zero() && new_value.is_zero() {
            return SSTORE_CLEAR;
        }else{
            return 0;
        }
    }

    if status.original == status.current {
        if !status.original.is_zero() && new_value.is_zero() {
            sstore_clear_schedule(revision)
        }else{
            0
        }
    } else {
        let mut refund = 0i64;
        if !status.original.is_zero() {
            if status.current.is_zero() {
                refund -= sstore_clear_schedule(revision);
            }
            if new_value.is_zero() {
                refund += sstore_clear_schedule(revision);
            }
        }
        if status.original == new_value {
            if status.original.is_zero() {
                refund += SSTORE_SET_GAS - sload_gas(revision);
            }else{
                refund += sstore_reset_gas(revision) - sload_gas(revision);
            }
        }
        refund
    }
}

const SSTORE_CLEAR: i64 = 15_000;
const SSTORE_SET_GAS: i64 = 20000;
const SSTORE_RESET_GAS: i64 = 5000;
const ACCESS_LIST_STORAGE_KEY_COST: i64 = 1900;

fn sload_gas(revision: Revision) -> i64 {
    if revision >= Revision::Berlin {
        100
    }else{
        match revision {
            Revision::Istanbul => 800,
            Revision::Constantinople => 200,
            _ => 0
        }
    }
}

fn sstore_reset_gas(revision: Revision) -> i64 {
    SSTORE_RESET_GAS + match revision {
        Revision::Berlin | Revision::London | Revision::Shanghai => -2100,
        _ => 0,
    }
}

fn sstore_clear_schedule(revision: Revision) -> i64 {
    // https://eips.ethereum.org/EIPS/eip-3529
    if revision >= Revision::London {
        sstore_reset_gas(revision) + ACCESS_LIST_STORAGE_KEY_COST
    }else{
        SSTORE_CLEAR
    }
}