use bytes::Bytes;
use ethereum_types::{U256, Address};
use evmstar::executor::callstack::CallScope;
use hex::decode;

use evmstar::host::stateful::{
    StatefulHost,
};
use evmstar::executor::{
    executor::Executor,
};
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

fn default_address() -> Address { Address::from_low_u64_be(0xffffeeee) }
fn consumed_gas(amount: i64) -> i64 {
    i64::max_value() - amount
}
fn get_address(address_hex: &str) -> Address {
    let mut dst = [0u8; 20];
    let hex = decode(address_hex).unwrap();
    for i in 0..hex.len() {
        dst[hex.len() - 1 - i] = hex[hex.len() - 1 - i];
    }
    let address = Address::from_slice(&dst);
    address
}

fn get_default_context() -> TxContext {
    TxContext {
        gas_price: U256::from(0x1234),
        origin: Address::from_low_u64_be(0x1234),
        coinbase: Address::from_low_u64_be(0xabcd),
        block_number: 0x1111,
        block_timestamp: 0x2222,
        gas_limit: 100_000,
        base_fee: U256::from(0x4444),
        chain_id: U256::from(0x01),
        difficulty: U256::from(0x5555),
    }
}
fn get_code_for_call(gas: i64, address: &str, value: usize, args_offset: u8, args_size: u8, ret_offset: u8, ret_size: u8) -> Code {
    let address = get_address(address);

    let mut builder = Code::builder();
    let code = builder
        .append(OpCode::PUSH1)
        .append(ret_size)
        .append(OpCode::PUSH1)
        .append(ret_offset)
        .append(OpCode::PUSH1)
        .append(args_size)
        .append(OpCode::PUSH1)
        .append(args_offset)
        .append(OpCode::PUSH32)
        .append(U256::from(value))
        .append(OpCode::PUSH20)
        .append(address)
        .append(OpCode::PUSH32)
        .append(U256::from(gas))
        .append(OpCode::CALL);
    code.clone()
}

#[test]
fn test_call() {
    let mut host = StatefulHost::new_with(get_default_context());

    let contract_address = "cc000001";
    let mut builder = Code::builder();
    let contract = builder
        .append(OpCode::PUSH1)
        .append(1)
        .append(OpCode::PUSH1)
        .append(2)
        .append(OpCode::ADD)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::RETURN)
        .clone();

    host.debug_deploy_contract(contract_address, contract, U256::zero());
    
    let mut builder = Code::builder();
    let code = builder
        .append(OpCode::PUSH1)
        .append(1)
        .append(OpCode::PUSH1)
        .append(1)
        .append_code(&mut get_code_for_call(50_000, contract_address, 0, 0, 0, 0, 32))
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::RETURN);
    
    let mut scope = CallScope::default();
    scope.code = code.clone();
    scope.to = default_address();
    
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let output = executor.execute_raw_with(scope);
    let data = decode("0000000000000000000000000000000000000000000000000000000000000003").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
}