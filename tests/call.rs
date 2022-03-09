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
fn consumed_gas(amount: i64, gas_limit: i64) -> i64 {
    gas_limit - amount
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
    code.clone()    // 3 * 7 + 2600 = 2621
}

#[test]
fn test_call() {
    let mut host = StatefulHost::new_with(get_default_context());

    let contract_address = "cc000001"; // 0xcc00000100000000000000000000000000000000 
    let mut builder = Code::builder();
    let contract = builder
        .append(OpCode::PUSH1)  // 3
        .append(0x00)
        .append(OpCode::CALLDATALOAD)   // 3
        .append(OpCode::PUSH1)  // 3
        .append(2)
        .append(OpCode::ADD)    // 3
        .append(OpCode::PUSH1)  // 3
        .append(0x00)
        .append(OpCode::MSTORE) // 3 + 3 = 6
        .append(OpCode::PUSH1)  // 3
        .append(0x20)
        .append(OpCode::PUSH1)  // 3
        .append(0x00)
        .append(OpCode::RETURN) // 0
        .clone(); // = 27

    host.debug_deploy_contract(contract_address, contract, U256::zero());
    
    let mut builder = Code::builder();
    let code = builder
        .append(OpCode::PUSH1)  // 3
        .append(0xa0)
        .append(OpCode::PUSH1)  // 3
        .append(0x00)
        .append(OpCode::MSTORE) // 3 + 3 = 6
        .append_code(&mut get_code_for_call(50_000, contract_address, 0, 0, 0x20, 0, 0x20))
            // call cost = 2621
            // execution cost = 27
            // = 2648
        .append(OpCode::PUSH1)  // 3
        .append(0x20)
        .append(OpCode::PUSH1)  // 3
        .append(0x00)
        .append(OpCode::RETURN) // 0
        .clone();
    
    let gas_limit = 100_000;
    let mut scope = CallScope::default();
    scope.code = code;
    scope.to = default_address();
    scope.gas_limit = gas_limit;
    scope.gas_left = gas_limit;
    
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let output = executor.execute_raw_with(scope);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000a2").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(2666, consumed_gas(output.gas_left, gas_limit));
}

#[test]
fn test_remote_self_balance() {
    let mut host = StatefulHost::new_with(get_default_context());
    let contract_address = "cc000001"; // 0xcc00000100000000000000000000000000000000

    let contract_balance = U256::from(0xabab12);
    let mut builder = Code::builder();
    let contract = builder
        .append(OpCode::SELFBALANCE)    // get contract balance
        .append("6000")
        .append(OpCode::MSTORE)
        .append("60206000")
        .append(OpCode::RETURN)
        .clone();   // = 20
    host.debug_deploy_contract(contract_address, contract, contract_balance);
    
    let mut builder = Code::builder();
    let code = builder
        .append_code(&mut get_code_for_call(50_000, contract_address, 0, 0, 0x00, 0, 0x20))
        // call cost = 2621 + 3(=memory expansion)
        // exec cost = 20
        // total = 2644
        .append("60206000") // 6
        .append(OpCode::RETURN)
        .clone();
    
    let gas_limit = 100_000;
    let mut scope = CallScope::default();
    scope.code = code;
    scope.to = default_address();
    scope.gas_limit = gas_limit;
    scope.gas_left = gas_limit;

    let mut executor = Executor::new_with_tracing(Box::new(host));
    let output = executor.execute_raw_with(scope);
    let data = decode("0000000000000000000000000000000000000000000000000000000000abab12").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(2650, consumed_gas(output.gas_left, gas_limit));
}

#[test]
fn test_remote_address() {
    let mut host = StatefulHost::new_with(get_default_context());
    let contract_address = "cc000001";

    let contract_balance = U256::from(0xabab12);
    let mut builder = Code::builder();
    let contract = builder
        .append(OpCode::ADDRESS)    // get contract address
        .append("6000")
        .append(OpCode::MSTORE)
        .append("60206000")
        .append(OpCode::RETURN)
        .clone();   // = 17
    host.debug_deploy_contract(contract_address, contract, contract_balance);
    
    let mut builder = Code::builder();
    let code = builder
        .append_code(&mut get_code_for_call(50_000, contract_address, 0, 0, 0x00, 0, 0x20))
        // call cost = 2621 + 3(=memory expansion)
        // exec cost = 17
        // total = 2641
        .append("60206000") // 6
        .append(OpCode::RETURN)
        .clone();
    
    let gas_limit = 100_000;
    let mut scope = CallScope::default();
    scope.code = code;
    scope.to = default_address();
    scope.gas_limit = gas_limit;
    scope.gas_left = gas_limit;

    let mut executor = Executor::new_with_tracing(Box::new(host));
    let output = executor.execute_raw_with(scope);
    let data = decode("000000000000000000000000cc00000100000000000000000000000000000000").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(2647, consumed_gas(output.gas_left, gas_limit));
}