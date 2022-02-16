use bytes::Bytes;
use ethereum_types::{U256, Address};

use evmstar::host::host::TransientHost;
use evmstar::executor::executor::Executor;
#[allow(unused_imports)]
use evmstar::model::{
    code::{Code},
    opcode::OpCode,
    evmc::{
        StatusCode, FailureKind,
        TxContext,
    },
};

use hex::{decode};

fn consumed_gas(amount: i64) -> i64 {
    i64::max_value() - amount
}

fn get_default_context() -> TxContext {
    TxContext {
        gas_price: U256::from(0x1234),
        origin: Address::from_low_u64_be(0x1234),
        coinbase: Address::from_low_u64_be(0xabcd),
        block_number: 0x1111,
        block_timestamp: 0x2222,
        gas_limit: 0x3333,
        base_fee: U256::from(0x4444),
        chain_id: U256::from(0x01),
        difficulty: U256::from(0x5555),
    }
}

#[test]
fn test_gas_price() {
    let host = TransientHost::new_with_context(get_default_context());
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::GASPRICE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("0000000000000000000000000000000000000000000000000000000000001234").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(17), output.gas_left);
}

#[test]
fn test_coinbase() {
    let host = TransientHost::new_with_context(get_default_context());
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::COINBASE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("000000000000000000000000000000000000000000000000000000000000abcd").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(17), output.gas_left);
}

#[test]
fn test_block_number() {
    let host = TransientHost::new_with_context(get_default_context());
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::NUMBER)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("0000000000000000000000000000000000000000000000000000000000001111").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(17), output.gas_left);
}

#[test]
fn test_block_timestamp() {
    let host = TransientHost::new_with_context(get_default_context());
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::TIMESTAMP)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("0000000000000000000000000000000000000000000000000000000000002222").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(17), output.gas_left);
}

#[test]
fn test_gas_limit() {
    let host = TransientHost::new_with_context(get_default_context());
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::GASLIMIT)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("0000000000000000000000000000000000000000000000000000000000003333").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(17), output.gas_left);
}

#[test]
fn test_base_fee() {
    let host = TransientHost::new_with_context(get_default_context());
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::BASEFEE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("0000000000000000000000000000000000000000000000000000000000004444").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(17), output.gas_left);
}

#[test]
fn test_chain_id() {
    let host = TransientHost::new_with_context(get_default_context());
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::CHAINID)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(17), output.gas_left);
}

#[test]
fn test_difficulty() {
    let host = TransientHost::new_with_context(get_default_context());
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::DIFFICULTY)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("0000000000000000000000000000000000000000000000000000000000005555").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(17), output.gas_left);
}

#[test]
fn test_blockhash() {
    let host = TransientHost::new_with_context(get_default_context());
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0x01])
        .append_opcode(OpCode::BLOCKHASH)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("0000000000000000000000000000000000000000000000000000000000000101").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(38), output.gas_left);
}