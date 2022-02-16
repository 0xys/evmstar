use bytes::Bytes;
use ethereum_types::{U256, Address};

use evmstar::host::stateful::{
    StatefulHost, Account,
};
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
fn test_extcodehash_cold() {
    let mut host = StatefulHost::new_with(get_default_context());
    let address = Address::from_low_u64_be(0x123456);
    let account = Account {
        nonce: 0,
        code: Bytes::default(),
        code_hash: U256::from(0xaa),
        balance: U256::from(0),
        storage: Default::default()
    };
    host.add_account(address, account);

    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH3)
        .append(&[0x12, 0x34, 0x56])
        .append_opcode(OpCode::EXTCODEHASH)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000aa").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(2618), output.gas_left);
}

#[test]
fn test_extcodehash_warm() {
    let mut host = StatefulHost::new_with(get_default_context());
    let address = Address::from_low_u64_be(0x123456);
    let account = Account {
        nonce: 0,
        code: Bytes::default(),
        code_hash: U256::from(0xaa),
        balance: U256::from(0),
        storage: Default::default()
    };
    host.add_account(address, account);

    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH3)
        .append(&[0x12, 0x34, 0x56])
        .append_opcode(OpCode::EXTCODEHASH)
        .append_opcode(OpCode::PUSH3)
        .append(&[0x12, 0x34, 0x56])
        .append_opcode(OpCode::EXTCODEHASH)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000aa").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(2721), output.gas_left);
}