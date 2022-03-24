use ethereum_types::{U256, Address};
use hex::decode;

use evmstar::tester::{
    EvmTester,
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

    let mut builder = Code::builder();
    let contract_address = "cc000001"; // 0xcc00000100000000000000000000000000000000 
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

    let mut tester = EvmTester::new_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .with_contract_deployed(contract_address, contract, U256::zero())
        .run_code(code);
    
    result.expect_status(StatusCode::Success)
        .expect_output("00000000000000000000000000000000000000000000000000000000000000a2")
        .expect_gas(2666);
}

#[test]
fn test_remote_self_balance() {
    let mut builder = Code::builder();
    let contract = builder
        .append(OpCode::SELFBALANCE)    // get contract balance
        .append("6000")
        .append(OpCode::MSTORE)
        .append("60206000")
        .append(OpCode::RETURN)
        .clone();   // = 20
    
    let contract_address = "cc000001"; // 0xcc00000100000000000000000000000000000000
    let contract_balance = U256::from(0xabab12);
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

    let mut tester = EvmTester::new_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .with_contract_deployed(contract_address, contract, contract_balance)
        .run_code(code);
    
    result.expect_status(StatusCode::Success)
        .expect_output("0000000000000000000000000000000000000000000000000000000000abab12")
        .expect_gas(2650);
}

#[test]
fn test_remote_address() {
    let mut builder = Code::builder();
    let contract = builder
        .append(OpCode::ADDRESS)    // get contract address
        .append("6000")
        .append(OpCode::MSTORE)
        .append("60206000")
        .append(OpCode::RETURN)
        .clone();   // = 17

    let contract_address = "cc000001";
    let contract_balance = U256::from(0xabab12);
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
    let mut tester = EvmTester::new_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .with_contract_deployed(contract_address, contract, contract_balance)
        .run_code(code);
    
    result.expect_status(StatusCode::Success)
        .expect_output("000000000000000000000000cc00000100000000000000000000000000000000")
        .expect_gas(2647);
}

fn scope_code(depth: u64) -> Code {
    let mut sstore = Code::builder()
        .append(OpCode::PUSH1)
        .append(U256::from(depth))  // value
        .append(OpCode::PUSH1)
        .append(0x00)   // key
        .append(OpCode::SSTORE)
        .clone();

    let mut scope_code = 
        if depth < 12 {
            Code::builder()
                .append(OpCode::PUSH1)
                .append(0x20)   // ret_size
                .append(OpCode::PUSH1)
                .append(0x00)   // ret_offset
                .append(OpCode::PUSH1)
                .append(0x00)   // args_size
                .append(OpCode::PUSH1)
                .append(0x00)   // args_offset
                .append(OpCode::PUSH32)
                .append(U256::from(0x00))   // value
                .append(OpCode::PUSH20)
                .append(Address::from_low_u64_be(depth))   // address
                .append(OpCode::GAS)
                .append(OpCode::CALL)
                .clone()
        } else {
            Code::builder()
                .append(0x20)
                .append(OpCode::PUSH1)
                .append(0x00)
                .append(OpCode::RETURN)
                .clone()
        };
    
    let code = Code::builder()
        .append_code(&mut sstore)
        .append_code(&mut scope_code)

        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::MLOAD)

        .append(OpCode::PUSH1)
        .append(U256::from(depth))
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
    code
}

#[test]
fn test_deep() {
    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append(0x20)   // ret_size
        .append(OpCode::PUSH1)
        .append(0x00)   // ret_offset
        .append(OpCode::PUSH1)
        .append(0x00)   // args_size
        .append(OpCode::PUSH1)
        .append(0x00)   // args_offset
        .append(OpCode::PUSH32)
        .append(U256::from(0x00))   // value
        .append(OpCode::PUSH20)
        .append(Address::from_low_u64_be(1))   // address
        .append(OpCode::GAS)
        // .append(OpCode::PUSH32)
        // .append(U256::from(100_000))    // gas
        .append(OpCode::CALL)
        .append(OpCode::PUSH1)
        .append(0x20)   // ret_size
        .append(OpCode::PUSH1)
        .append(0x00)   // ret_offset
        .append(OpCode::RETURN)
        .clone();
    
    let mut tester = EvmTester::new_with(get_default_context());
    let result = tester
        .with_default_gas()
        .with_contract_deployed2(Address::from_low_u64_be(1), scope_code(1), U256::zero())
        .with_contract_deployed2(Address::from_low_u64_be(2), scope_code(2), U256::zero())
        .with_contract_deployed2(Address::from_low_u64_be(3), scope_code(3), U256::zero())
        .with_contract_deployed2(Address::from_low_u64_be(4), scope_code(4), U256::zero())
        .with_contract_deployed2(Address::from_low_u64_be(5), scope_code(5), U256::zero())
        .with_contract_deployed2(Address::from_low_u64_be(6), scope_code(6), U256::zero())
        .with_contract_deployed2(Address::from_low_u64_be(7), scope_code(7), U256::zero())
        .with_contract_deployed2(Address::from_low_u64_be(8), scope_code(8), U256::zero())
        .with_contract_deployed2(Address::from_low_u64_be(9), scope_code(9), U256::zero())
        .with_contract_deployed2(Address::from_low_u64_be(10), scope_code(10), U256::zero())
        .with_contract_deployed2(Address::from_low_u64_be(11), scope_code(11), U256::zero())
        .with_contract_deployed2(Address::from_low_u64_be(12), scope_code(12), U256::zero())
        .run_code(code);

    
    result
        .expect_status(StatusCode::Success)
        // .expect_output("000000000000000000000000000000000000000000000000000000000000004e")  // 0x4e = 78 = sum(1..12)
        .expect_storage(Address::from_low_u64_be(1), U256::from(0x00), U256::from(1))
        // .expect_storage(Address::from_low_u64_be(2), U256::from(0x00), U256::from(2))
        // .expect_storage(Address::from_low_u64_be(3), U256::from(0x00), U256::from(3))
        // .expect_storage(Address::from_low_u64_be(4), U256::from(0x00), U256::from(4))
        // .expect_storage(Address::from_low_u64_be(5), U256::from(0x00), U256::from(5))
        // .expect_storage(Address::from_low_u64_be(6), U256::from(0x00), U256::from(6))
        // .expect_storage(Address::from_low_u64_be(7), U256::from(0x00), U256::from(7))
        // .expect_storage(Address::from_low_u64_be(8), U256::from(0x00), U256::from(8))
        // .expect_storage(Address::from_low_u64_be(9), U256::from(0x00), U256::from(9))
        // .expect_storage(Address::from_low_u64_be(10), U256::from(0x00), U256::from(10))
        // .expect_storage(Address::from_low_u64_be(11), U256::from(0x00), U256::from(11))
        // .expect_storage(Address::from_low_u64_be(12), U256::from(0x00), U256::from(12))
        ;
    
}