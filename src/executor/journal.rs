use ethereum_types::{
    Address, U256
};
use crate::host::Host;

#[derive(Clone, Debug, Default)]
pub struct Journal {
    pub storage_log: Vec<StorageDelta>,
}

pub type Snapshot = usize;

impl Journal {
    pub fn record_storage(&mut self, address: Address, key: U256, value: U256) {
        let delta = StorageDelta {
            address,
            key,
            previous: value,
        };
        self.storage_log.push(delta);
    }

    pub fn take_snapshot(&self) -> Snapshot {
        self.storage_log.len() - 1
    }

    pub fn rollback(&mut self, host: &mut dyn Host, snapshot: Snapshot) {
        let length = self.storage_log.len();
        for _ in 0..length - 1 - snapshot {
            if let Some(delta) = self.storage_log.pop() {
                host.force_set_storage(delta.address, delta.key, delta.previous);
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StorageDelta {
    pub address: Address,
    pub key: U256,
    pub previous: U256,
}

