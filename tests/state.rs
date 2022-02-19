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
use evmstar::interpreter::stack::Calldata;

#[allow(unused_imports)]
use evmstar::model::{
    code::{
        Code, Append,
    },
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
        .append("123456")
        .append_opcode(OpCode::EXTCODEHASH)
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append("20")
        .append_opcode(OpCode::PUSH1)
        .append("00")
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
        .append("123456")
        .append_opcode(OpCode::EXTCODEHASH)
        .append_opcode(OpCode::PUSH3)
        .append("123456")
        .append_opcode(OpCode::EXTCODEHASH)
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append("20")
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000aa").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(2721), output.gas_left);
}

#[test]
fn test_pre_eip1283_sstore() {
    for r in Revision::iter() {
        if r < Revision::Constantinople || r == Revision::Petersburg {
            test_sstore_of(r);
        }
    }
}

fn test_sstore_of(revision: Revision) {

    let original = 0x00;
    test_sstore_legacy_logic(hex!("60006000556000600055").to_vec(), 10012,     0, original, revision);
    test_sstore_legacy_logic(hex!("60006000556001600055").to_vec(), 25012,     0, original, revision);
    test_sstore_legacy_logic(hex!("60016000556000600055").to_vec(), 25012, 15000, original, revision);
    test_sstore_legacy_logic(hex!("60016000556002600055").to_vec(), 25012,     0, original, revision);
    test_sstore_legacy_logic(hex!("60016000556001600055").to_vec(), 25012,     0, original, revision);

    let original = 0x01;
    test_sstore_legacy_logic(hex!("60006000556000600055").to_vec(), 10012, 15000, original, revision);
    test_sstore_legacy_logic(hex!("60006000556001600055").to_vec(), 25012, 15000, original, revision);
    test_sstore_legacy_logic(hex!("60006000556002600055").to_vec(), 25012, 15000, original, revision);
    test_sstore_legacy_logic(hex!("60026000556000600055").to_vec(), 10012, 15000, original, revision);
    test_sstore_legacy_logic(hex!("60026000556003600055").to_vec(), 10012,     0, original, revision);

    test_sstore_legacy_logic(hex!("60026000556001600055").to_vec(), 10012,     0, original, revision);
    test_sstore_legacy_logic(hex!("60026000556002600055").to_vec(), 10012,     0, original, revision);
    test_sstore_legacy_logic(hex!("60016000556000600055").to_vec(), 10012, 15000, original, revision);
    test_sstore_legacy_logic(hex!("60016000556002600055").to_vec(), 10012,     0, original, revision);
    test_sstore_legacy_logic(hex!("60016000556001600055").to_vec(), 10012,     0, original, revision);

    test_sstore_legacy_logic(hex!("600160005560006000556001600055").to_vec(), 45018, 15000, 0x00, revision);
    test_sstore_legacy_logic(hex!("600060005560016000556000600055").to_vec(), 30018, 30000, 0x01, revision);
}

fn test_sstore_legacy_logic(code: Vec<u8>, gas_used: i64, gas_refund: i64, original: usize, revision: Revision) {
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append(code.as_slice());
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
    let code = builder.append(code.as_slice());
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, revision);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(gas_used, consumed_gas(output.gas_left));
}

#[test]
fn test_calldatasize() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();
    let code = builder.append("3660005260206000f3");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();
    context.calldata = Calldata::from("ffff");

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Shanghai);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("0000000000000000000000000000000000000000000000000000000000000002"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(17, consumed_gas(output.gas_left));
}

#[test]
fn test_calldataload() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();
    let code = builder.append("60003560005260206000f3");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();
    context.calldata = Calldata::from("ffff");

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Shanghai);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("ffff000000000000000000000000000000000000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(21, consumed_gas(output.gas_left));
}

#[test]
fn test_calldataload_2() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();

    // 7feeee00000000000000000000000000000000000000000000000000000000000060005260003560105260206000f3
    let code = builder
        .append_opcode(OpCode::PUSH32)
        .append("eeee000000000000000000000000000000000000000000000000000000000000")
        .append_opcode(OpCode::PUSH1)
        .append(0)
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(0)
        .append_opcode(OpCode::CALLDATALOAD)
        .append("60105260206000f3");

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();
    context.calldata = Calldata::from("ffff");

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Shanghai);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("eeee0000000000000000000000000000ffff0000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(33, consumed_gas(output.gas_left));
}

#[test]
fn test_calldatacopy() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();

    // 6020600060003760206000f3
    let code = builder
        .append("602060006000")
        .append_opcode(OpCode::CALLDATACOPY)
        .append("60206000")
        .append_opcode(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();
    context.calldata = Calldata::from("ffff");

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Berlin);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("ffff000000000000000000000000000000000000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(24, consumed_gas(output.gas_left));
}

#[test]
fn test_calldatacopy_2() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();

    // 7feeee0000000000000000000000000000000000000000000000000000000000006000526020600060103760206000f3
    let code = builder
        .append_opcode(OpCode::PUSH32)
        .append("eeee000000000000000000000000000000000000000000000000000000000000")
        .append_opcode(OpCode::PUSH1)
        .append(0)
        .append_opcode(OpCode::MSTORE)
        .append("602060006010")
        .append_opcode(OpCode::CALLDATACOPY)
        .append("60206000")
        .append_opcode(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();
    context.calldata = Calldata::from("ffff");

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Berlin);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("eeee0000000000000000000000000000ffff0000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(36, consumed_gas(output.gas_left));
}

#[test]
fn test_codesize() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();

    // 3860005260206000f3
    let code = builder
        .append_opcode(OpCode::CODESIZE)
        .append("6000")
        .append_opcode(OpCode::MSTORE)
        .append("60206000")
        .append_opcode(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Berlin);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("0000000000000000000000000000000000000000000000000000000000000009"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(17, consumed_gas(output.gas_left));
}

#[test]
fn test_codecopy() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();

    // 600c600060003960206000f3
    let code = builder
        .append("600c60006000")
        .append_opcode(OpCode::CODECOPY)
        .append("60206000")
        .append_opcode(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Berlin);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("600c600060003960206000f30000000000000000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(24, consumed_gas(output.gas_left));
}

#[test]
fn test_codecopy_out_of_bounds() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();

    // 6040600060003960406000f3
    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(64) // out of bounds
        .append("60006000")
        .append_opcode(OpCode::CODECOPY)
        .append("60406000")
        .append_opcode(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Berlin);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("6040600060003960406000f300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(30, consumed_gas(output.gas_left));
}