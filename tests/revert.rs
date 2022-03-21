use ethereum_types::{U256, Address};
use evmstar::tester::EvmTester;
use evmstar::model::{
    code::{
        Code, Append,
    },
    opcode::OpCode,
    evmc::{
        StatusCode, FailureKind,
        TxContext,
    }
};

fn default_address() -> Address { Address::from_low_u64_be(0xffffeeee) }

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

#[test]
fn test_revert_one_level() {
    let mut builder = Code::builder();
    let code = builder
        .append(OpCode::PUSH1)  // 3
        .append(0xdd)           // data
        .append(OpCode::PUSH1)  // 3
        .append(0x00)           // offset
        .append(OpCode::SSTORE) // 20000 + 2100
        .append("60aa60005260206000") // 3*4 + 6 = 18
        .append(OpCode::REVERT) // 0
        .clone(); // = 22124
    
    let gas_limit = 100_000;
    
    let mut tester = EvmTester::new_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .run_code(code);
    
    result.expect_status(StatusCode::Failure(FailureKind::Revert))
        .expect_output("00000000000000000000000000000000000000000000000000000000000000aa")
        .expect_gas(22124)
        .expect_storage(default_address(), U256::from(0x01), U256::from(0x00));
}