use bytes::Bytes;
use ethereum_types::{U256, Address};
use hex_literal::hex;

use evmstar::host::stateful::{
    StatefulHost, Account,
};
use evmstar::executor::{
    callstack::CallContext,
    executor::Executor,
};
#[allow(unused_imports)]
use evmstar::model::{
    code::{Code},
    opcode::OpCode,
    evmc::{
        StatusCode, FailureKind,
        TxContext,
    },
    revision::Revision,
};

use hex::{decode};

fn default_address() -> Address { Address::from_low_u64_be(0xffffeeee) }

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

#[test]
fn test_storage_legacy() {
    test_storage(hex!("60006000556000600055").to_vec(), 10012, 0, 0, Revision::Homestead);
    test_storage(hex!("60006000556000600055").to_vec(), 10012, 15000, 0x01, Revision::Homestead);
    test_storage(hex!("60006000556000600055").to_vec(), 10012, 0, 0x00, Revision::Homestead);
    test_storage(hex!("60006000556000600055").to_vec(), 10012, 15000, 0x01, Revision::Homestead);

    test_storage(hex!("600160005560006000556000600055").to_vec(), 30018, 15000, 0x00, Revision::Homestead);
    test_storage(hex!("600160005560006000556000600055").to_vec(), 15018, 15000, 0x01, Revision::Homestead);
    test_storage(hex!("600060005560016000556000600055").to_vec(), 30018, 15000, 0x00, Revision::Homestead);
    test_storage(hex!("600060005560016000556000600055").to_vec(), 15018, 30000, 0x01, Revision::Homestead);
    test_storage(hex!("600060005560006000556001600055").to_vec(), 30018,     0, 0x00, Revision::Homestead);
    test_storage(hex!("600060005560006000556001600055").to_vec(), 15018, 15000, 0x01, Revision::Homestead);

    test_storage(hex!("6000600055600160005560006000556001600055").to_vec(), 35024, 15000, 0x00, Revision::Homestead);
    test_storage(hex!("6000600055600160005560006000556001600055").to_vec(), 20024, 30000, 0x01, Revision::Homestead);
}

fn test_storage(code: Vec<u8>, gas_used: i64, gas_refund: i64, original: usize, revision: Revision) {
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append(&code);
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, revision);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(gas_used, consumed_gas(output.gas_left));
    assert_eq!(gas_refund, output.gas_refund);
}

#[test]
fn test_sload() {
    let code = &hex!("60006000546000600054").to_vec();
    test_sload_logic(code, 112, Revision::Frontier);
    test_sload_logic(code, 112, Revision::Homestead);
    test_sload_logic(code, 412, Revision::Tangerine);
    test_sload_logic(code, 412, Revision::Spurious);
    test_sload_logic(code, 412, Revision::Byzantium);
    test_sload_logic(code, 412, Revision::Constantinople);
    test_sload_logic(code, 412, Revision::Petersburg);
    test_sload_logic(code, 1612, Revision::Istanbul);
    test_sload_logic(code, 2212, Revision::Berlin);
    test_sload_logic(code, 2212, Revision::London);
    test_sload_logic(code, 2212, Revision::Shanghai);
}

fn test_sload_logic(code: &Vec<u8>, gas_used: i64, revision: Revision) {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();
    let code = builder.append(code);
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, revision);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(gas_used, consumed_gas(output.gas_left));
}