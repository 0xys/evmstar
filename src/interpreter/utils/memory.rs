use ethereum_types::U256;

use crate::interpreter::stack::{
    Memory, Stack
};
use crate::model::evmc::StatusCode;

const g_memory: i64 = 3;

/// load one word in memory starting from the `offset`.
/// if offset + size is not a multiple of word size, new memory region is allocated to pad the gap.
/// gas cost = g_memory * a * a^2/512, where a is number of bytes newly allocated.
/// as defined in equation (326) in yellow paper.
pub fn mload(offset: U256, memory: &mut Memory, stack: &mut Stack, gas_left: i64) -> Result<i64, StatusCode> {
    Ok(0)
}

/// store the top item of the stack into memory at `offset`.
/// any padding occurred will incur gas cost.
pub fn mstore(offset: U256, memory: &mut Memory, stack: &mut Stack, gas_left: i64) -> Result<i64, StatusCode> {
    Ok(0)
}

/// store the top item of the stack into memory at `offset`.
/// any padding occurred will incur gas cost.
pub fn mstore8(offset: U256, memory: &mut Memory, stack: &mut Stack, gas_left: i64) -> Result<i64, StatusCode> {
    Ok(0)
}