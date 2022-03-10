use std::collections::HashMap;

use ethereum_types::U256;

#[derive(Clone, Debug, Default)]
pub struct StorageValue {
    pub original: U256,
    pub current: U256,
    pub modified: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ScopedStorage {
    mapping: HashMap<U256, StorageValue>
}

impl ScopedStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_or_modify(&mut self, key: U256, new_value: U256) {
        let value = self.mapping.entry(key).or_default();
        if !value.modified {
            value.modified = true;
            value.original = new_value;
        }
        value.current = new_value;
    }

    pub fn revert(&mut self, key: U256) {
        let value = self.mapping.entry(key).or_default();
        if value.modified {
            value.current = value.original;
        }
    }

    pub fn revert_all(&mut self) {
        let all_keys: Vec<U256> = self.mapping
            .iter()
            .map(|x| *x.0)
            .collect();
        for k in all_keys {
            self.revert(k);
        }
    }
}