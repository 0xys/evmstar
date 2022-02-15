use crate::model::{
    revision::Revision,
};

use crate::model::evmc::{
    AccessStatus, StorageStatus
};


pub fn calc_sstore_gas_cost(revision: Revision, access_status: AccessStatus, storage_status: StorageStatus) -> i64 {
    0
}

pub fn calc_sstore_gas_refund(revision: Revision, access_status: AccessStatus, storage_status: StorageStatus) -> i64 {
    0
}