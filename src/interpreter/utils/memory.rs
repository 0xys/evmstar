use ethereum_types::U256;

use crate::interpreter::stack::{
    Memory, Stack, num_words, MAX_BUFFER_SIZE, WORD_SIZE
};
use crate::model::evmc::{
    StatusCode, FailureKind
};

const G_MEMORY: i64 = 3;

/// load one word in memory starting from the `offset`.
/// if offset + size is not a multiple of word size, new memory region is allocated to pad the gap.
/// gas cost = g_memory * a * a^2/512, where a is number of bytes newly allocated.
/// as defined in equation (326) in yellow paper.
pub fn mload(offset: U256, memory: &mut Memory, stack: &mut Stack, gas_left: i64) -> Result<i64, StatusCode> {
    if offset > U256::from(MAX_BUFFER_SIZE) {
        return Err(StatusCode::Failure(FailureKind::ArgumentOutOfRange));
    }
    let offset = offset.as_usize();

    let gas_consumed = try_expand_memory(offset, WORD_SIZE as usize, memory, gas_left)?;
    let word = memory.get_word(offset);
    stack.push(word);
    
    Ok(gas_consumed)
}

/// store the top item of the stack into memory at `offset`.
/// any padding occurred will incur gas cost.
pub fn mstore(offset: U256, memory: &mut Memory, stack: &mut Stack, gas_left: i64) -> Result<i64, StatusCode> {
    if offset > U256::from(MAX_BUFFER_SIZE) {
        return Err(StatusCode::Failure(FailureKind::ArgumentOutOfRange));
    }
    let offset = offset.as_usize();
    let gas_consumed = try_expand_memory(offset, WORD_SIZE as usize, memory, gas_left)?;

    let top = stack.pop().map_err(|_| StatusCode::Failure(FailureKind::StackUnderflow))?;
    let mut word = [0u8; 32];
    top.to_big_endian(&mut word);
    memory.set_range(offset, &word);

    Ok(gas_consumed)
}

/// store the top item of the stack into memory at `offset`.
/// any padding occurred will incur gas cost.
pub fn mstore8(offset: U256, memory: &mut Memory, stack: &mut Stack, gas_left: i64) -> Result<i64, StatusCode> {
    if offset > U256::from(MAX_BUFFER_SIZE) {
        return Err(StatusCode::Failure(FailureKind::ArgumentOutOfRange));
    }
    let offset = offset.as_usize();
    let gas_consumed = try_expand_memory(offset, 1usize, memory, gas_left)?;

    let top = stack.pop().map_err(|_| StatusCode::Failure(FailureKind::StackUnderflow))?;
    memory.set(offset, (top.low_u32() & 0xff) as u8);

    Ok(gas_consumed)
}

fn try_expand_memory(offset: usize, size: usize, memory: &mut Memory, gas_left: i64) -> Result<i64, StatusCode> {
    let new_size = offset + size;
    let current_size: usize = memory.0.len();

    if current_size >= new_size {
        return Ok(0i64);    // resize didn't occur. no gas.
    }

    let new_num_of_words = num_words(new_size);
    let current_num_of_words = current_size as i64 / WORD_SIZE;
    
    let new_cost = func_326(new_num_of_words);
    let current_cost = func_326(current_num_of_words);
    let cost = new_cost - current_cost;

    if gas_left - cost < 0 {
        return Err(StatusCode::Failure(FailureKind::OutOfGas));
    }

    memory.0.resize((new_num_of_words * WORD_SIZE) as usize, Default::default());
    Ok(cost)
}

#[inline(always)]
fn func_326(num_of_words: i64) -> i64 {
    G_MEMORY * num_of_words + num_of_words*num_of_words/512
}