use ethereum_types::U256;

use crate::model::{
    revision::Revision,
};

use crate::model::evmc::{
    AccessStatus, StorageDiff,
};

/// calculate gas cost
/// 
/// sstore on Constantinople: https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1283.md
/// 
/// sstore on Istanbul: https://github.com/sorpaas/EIPs/blob/sp-eip-new-net-metering/EIPS/eip-2200.md
/// 
/// sstore on Berlin: https://eips.ethereum.org/EIPS/eip-2929
pub fn calc_sstore_gas_cost(new_value: U256, revision: Revision, access_status: AccessStatus, diff: StorageDiff) -> i64 {    
    // unchanged: current == new_value
    if diff.current == new_value {
        return match (revision, access_status) {
            (Revision::Berlin, AccessStatus::Cold) => 2200,
            (Revision::Berlin, AccessStatus::Warm) => 100,
            (Revision::Istanbul, _) => 800,
            (Revision::Constantinople, _) => 200,
            _ => 5000
        };
    }
    
    if diff.original == diff.current {
        if diff.original.is_zero() {
            20000 + match (revision, access_status) {
                (Revision::Berlin, AccessStatus::Cold) => 2100,
                (Revision::Berlin, AccessStatus::Warm) => 0,
                _ => 0,
            }
        }else{
            5000 + match (revision, access_status) {
                (Revision::Berlin, AccessStatus::Cold) => 0,
                (Revision::Berlin, AccessStatus::Warm) => -2100,
                _ => 0,
            }
        }
    }else{
        match (revision, access_status) {
            (Revision::Berlin, AccessStatus::Cold) => 2200,
            (Revision::Berlin, AccessStatus::Warm) => 100,
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
pub fn calc_sstore_gas_refund(new_value: U256, revision: Revision, diff: StorageDiff) -> i64 {
    let is_eip1283 = revision >= Revision::Istanbul || revision == Revision::Constantinople;
    
    if !is_eip1283 {
        if new_value.is_zero() {
            return SSTORE_CLEAR;
        }
    }

    if diff.original == diff.current {
        if diff.original.is_zero() {
            0
        }else{
            sstore_clear_schedule(revision)
        }
    }else{
        if !diff.original.is_zero() {
            if diff.current.is_zero() {
                return -sstore_clear_schedule(revision);
            }
            if new_value.is_zero() {
                return sstore_clear_schedule(revision);
            }
            0
        }else{
            // reset
            if diff.original == new_value {
                if diff.original.is_zero() {
                    return SSTORE_SET_GAS - sload_gas(revision);
                }else{
                    return sstore_reset_gas(revision) - sload_gas(revision);
                }
            }
            0
        }
    }
}

const SSTORE_CLEAR: i64 = 15_000;
const SSTORE_SET_GAS: i64 = 20000;
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
    5000 + match revision {
        Revision::Berlin => -2100,
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