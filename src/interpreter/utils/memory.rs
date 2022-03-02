use ethereum_types::U256;
use bytes::Bytes;

use crate::interpreter::stack::{
    Memory, Stack, num_words, MAX_BUFFER_SIZE, WORD_SIZE
};
use crate::model::evmc::{
    FailureKind
};

const G_MEMORY: i64 = 3;

/// load one word in memory starting from the `offset`.
/// if offset + size is not a multiple of word size, new memory region is allocated to pad the gap.
/// gas cost = g_memory * a * a^2/512, where a is number of bytes newly allocated.
/// as defined in equation (326) in yellow paper.
pub fn mload(offset: U256, memory: &mut Memory, stack: &mut Stack, gas_left: i64) -> Result<i64, FailureKind> {
    if offset > U256::from(MAX_BUFFER_SIZE) {
        return Err(FailureKind::ArgumentOutOfRange);
    }
    let offset = offset.as_usize();

    let gas_consumed = try_expand_memory(offset, WORD_SIZE as usize, memory, gas_left)?;
    let word = memory.get_word(offset);
    stack.push_unchecked(word);
    
    Ok(gas_consumed)
}

/// store the top item of the stack into memory at `offset`.
/// any padding added will incur gas cost.
pub fn mstore(offset: U256, memory: &mut Memory, stack: &mut Stack, gas_left: i64) -> Result<i64, FailureKind> {
    if offset > U256::from(MAX_BUFFER_SIZE) {
        return Err(FailureKind::ArgumentOutOfRange);
    }
    let offset = offset.as_usize();
    let gas_consumed = try_expand_memory(offset, WORD_SIZE as usize, memory, gas_left)?;

    let top = stack.pop()?;
    let mut word = [0u8; 32];
    top.to_big_endian(&mut word);
    memory.set_range(offset, &word);

    Ok(gas_consumed)
}

/// store the top item of the stack into memory at `offset`.
/// any padding added will incur gas cost.
pub fn mstore8(offset: U256, memory: &mut Memory, stack: &mut Stack, gas_left: i64) -> Result<i64, FailureKind> {
    if offset > U256::from(MAX_BUFFER_SIZE) {
        return Err(FailureKind::ArgumentOutOfRange);
    }
    let offset = offset.as_usize();
    let gas_consumed = try_expand_memory(offset, 1usize, memory, gas_left)?;

    let top = stack.pop()?;
    memory.set(offset, (top.low_u32() & 0xff) as u8);

    Ok(gas_consumed)
}

/// store variable-sized byte array into memory at `offset`
/// any padding added will incur gas cost.
/// return dynamic part of the cost.
pub fn mstore_data(offset: U256, memory: &mut Memory, data: &[u8], gas_left: i64) -> Result<i64, FailureKind> {
    if offset > U256::from(MAX_BUFFER_SIZE) {
        return Err(FailureKind::ArgumentOutOfRange);
    }

    let min_word_size = (data.len() + 31) / 32;

    let offset = offset.as_usize();
    let expansion_cost = try_expand_memory(offset, data.len(), memory, gas_left)?;
    let word_cost: i64 = 3 * min_word_size as i64;

    memory.set_range(offset, data);

    Ok(expansion_cost + word_cost)
}

/// return value of `size` at `offset` in memory.
/// it incurs memory expansion cost.
pub fn ret(offset: U256, size: U256, memory: &mut Memory, gas_left: i64) -> Result<(i64, Bytes), FailureKind> {
    if offset > U256::from(MAX_BUFFER_SIZE) {
        return Err(FailureKind::ArgumentOutOfRange);
    }
    let offset = offset.as_usize();

    if size.is_zero() {
        return Ok((0, Bytes::default()));
    }
    if size > U256::from(MAX_BUFFER_SIZE) {
        return Err(FailureKind::ArgumentOutOfRange);
    }
    let size = size.as_usize();

    let gas_consumed = try_expand_memory(offset, size, memory, gas_left)?;
    let data = memory.get_range(offset, size);

    Ok((gas_consumed, Bytes::from(data.to_owned())))
}

pub fn resize_memory(offset: usize, size: usize, memory: &mut Memory, gas_left: i64) -> Result<i64, FailureKind> {
    try_expand_memory(offset, size, memory, gas_left)
}

/// memory is resized as needed and calculate memory expansion cost.
/// 
fn try_expand_memory(offset: usize, size: usize, memory: &mut Memory, gas_left: i64) -> Result<i64, FailureKind> {
    let new_size = offset + size;
    let current_size: usize = memory.0.len();

    if current_size >= new_size {
        return Ok(0i64);    // resize doesn't occur. no gas.
    }

    let new_num_of_words = num_words(new_size);
    let current_num_of_words = current_size as i64 / WORD_SIZE;
    
    let new_cost = func_326(new_num_of_words);
    let current_cost = func_326(current_num_of_words);
    let cost = new_cost - current_cost;

    if gas_left - cost < 0 {
        return Err(FailureKind::OutOfGas);
    }

    memory.0.resize((new_num_of_words * WORD_SIZE) as usize, Default::default());
    Ok(cost)
}

#[inline(always)]
fn func_326(num_of_words: i64) -> i64 {
    G_MEMORY * num_of_words + num_of_words*num_of_words/512
}

#[test]
fn test_memory_expansion(){
    {
        let offset = 0;
        let size = 1;
        let mut memory = Memory::default();
        let gas_left = i64::max_value();
    
        let result = try_expand_memory(offset, size, &mut memory, gas_left);
        assert_eq!(true, result.is_ok());
        let gas_consumed = result.unwrap();
        assert_eq!(3, gas_consumed);
        assert_eq!(32, memory.0.len())
    }

    {
        let offset = 31;
        let size = 1;
        let mut memory = Memory::default();
        let gas_left = i64::max_value();
    
        let result = try_expand_memory(offset, size, &mut memory, gas_left);
        assert_eq!(true, result.is_ok());
        let gas_consumed = result.unwrap();
        assert_eq!(3, gas_consumed);
        assert_eq!(32, memory.0.len())
    }

    {
        let offset = 31;
        let size = 2;
        let mut memory = Memory::default();
        let gas_left = i64::max_value();
    
        let result = try_expand_memory(offset, size, &mut memory, gas_left);
        assert_eq!(true, result.is_ok());
        let gas_consumed = result.unwrap();
        assert_eq!(6, gas_consumed);
        assert_eq!(32*2, memory.0.len())
    }

    {   // no expansion
        let mut memory = Memory::default();
        memory.0.resize((10 * WORD_SIZE) as usize, Default::default());

        let offset = 0;
        let size = 2;
        let gas_left = i64::max_value();
    
        let result = try_expand_memory(offset, size, &mut memory, gas_left);
        assert_eq!(true, result.is_ok());
        let gas_consumed = result.unwrap();
        assert_eq!(0, gas_consumed);
        assert_eq!((10 * WORD_SIZE) as usize, memory.0.len())
    }

    {
        let offset = 600;
        let size = 1200;
        let mut memory = Memory::default();
        let gas_left = i64::max_value();
    
        let result = try_expand_memory(offset, size, &mut memory, gas_left);
        assert_eq!(true, result.is_ok());
        let gas_consumed = result.unwrap();
        let num_of_words = (offset + size + WORD_SIZE as usize) as i64 / WORD_SIZE;
        let expected = 177;
        assert_eq!(expected, gas_consumed);
        assert_eq!((num_of_words * WORD_SIZE) as usize, memory.0.len())
    }

    {   // not enough gas
        let offset = 600;
        let size = 1200;
        let mut memory = Memory::default();
        let gas_left = 176;
    
        let result = try_expand_memory(offset, size, &mut memory, gas_left);
        assert_eq!(false, result.is_ok());
    }
}