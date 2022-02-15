use crate::model::{
    revision::Revision,
};

use crate::model::evmc::{
    AccessStatus, StorageStatus
};


pub fn calc_sstore_gas_cost(revision: Revision, access_status: AccessStatus, storage_status: StorageStatus) -> i64 {    
    if revision >= Revision::Berlin {
        match (storage_status, access_status) {
            //  0 -> 1
            (StorageStatus::Added, AccessStatus::Warm) => 20000,
            (StorageStatus::Added, AccessStatus::Cold) => 22100,

            // 1 -> 2, 1 -> 0
            (StorageStatus::Modified | StorageStatus::Deleted, AccessStatus::Warm) => 2900,
            (StorageStatus::Modified | StorageStatus::Deleted, AccessStatus::Cold) => 5000,

            // 2 -> 3 [-> and so on]
            (StorageStatus::ModifiedAgain, _) => 100,

            // 0,1 -> 0,1
            (StorageStatus::Unchanged, AccessStatus::Warm) => 100,
            (StorageStatus::Unchanged, AccessStatus::Cold) => 2200,
        }
    }else{
        match storage_status {
            StorageStatus::Added => 20000,
            StorageStatus::Unchanged | StorageStatus::ModifiedAgain => match revision {
                Revision::Istanbul => 800,          // https://github.com/ethereum/EIPs/blob/master/EIPS/eip-2200.md
                Revision::Constantinople => 200,    // https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1283.md
                _ => 5000
            },
            StorageStatus::Modified | StorageStatus::Deleted => 5000
        }
    }
}

pub fn calc_sstore_gas_refund(revision: Revision, access_status: AccessStatus, storage_status: StorageStatus) -> i64 {
    0
}