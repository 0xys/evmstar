use ethereum_types::{U256, Address};
use evmstar::model::{
    code::{
        Code, Append,
    },
    opcode::OpCode,
    evmc::{
        StatusCode,
        TxContext,
    },
    revision::Revision,
};
use evmstar::emulator::EvmEmulator;

fn address_0() -> Address {
    Address::from_low_u64_be(0)
}
fn address_1() -> Address {
    Address::from_low_u64_be(1)
}
fn address_2() -> Address {
    Address::from_low_u64_be(2)
}
fn address_default() -> Address {
    Address::from_low_u64_be(0xaaaaaaaa)
}
fn address_ext() -> Address {
    Address::from_low_u64_be(0xffffffff)
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
fn test_eip2930_nocode() {
    {
        let mut tester = EvmEmulator::new_with(get_default_context());
        let result = tester
            .with_default_gas()
            .add_accessed_account(address_0())
            .enable_execution_cost()
            .run_as(Revision::Berlin);
        
        result.expect_status(StatusCode::Success)
            .expect_output("")
            .expect_gas(21000 + 2400);
    }
    {
        let mut tester = EvmEmulator::new_with(get_default_context());
        let result = tester
            .with_default_gas()
            .add_accessed_account(address_0())
            .add_accessed_account(address_0())  // dupulicate costs
            .add_accessed_account(address_1())
            .enable_execution_cost()
            .run_as(Revision::Berlin);
        
        result.expect_status(StatusCode::Success)
            .expect_output("")
            .expect_gas(21000 + 2400*3);
    }
}

#[test]
fn test_eip2930_nocode_storage() {
    {
        let mut tester = EvmEmulator::new_with(get_default_context());
        let result = tester
            .with_default_gas()
            .add_accessed_account(address_0())
            .add_accessed_account(address_0())  // dupulicate costs
            .add_accessed_account(address_1())
            .add_accessed_storage(address_0(), U256::zero())
            .enable_execution_cost()
            .run_as(Revision::Berlin);
        
        result.expect_status(StatusCode::Success)
            .expect_output("")
            .expect_gas(21000 + 2400*3 + 1900);
    }
    {
        let mut tester = EvmEmulator::new_with(get_default_context());
        let result = tester
            .with_default_gas()
            .add_accessed_account(address_0())
            .add_accessed_account(address_0())  // dupulicate costs
            .add_accessed_account(address_1())
            .add_accessed_storage(address_0(), U256::zero())
            .add_accessed_storage(address_2(), U256::zero())
            .enable_execution_cost()
            .run_as(Revision::Berlin);
        
        result.expect_status(StatusCode::Success)
            .expect_output("")
            .expect_gas(21000 + 2400*4 + 1900*2);
    }
}

#[test]
fn test_eip2930() {
    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append(0)
        .append(OpCode::SLOAD)  // warm

        .append(OpCode::PUSH1)
        .append(1)
        .append(OpCode::SLOAD)   // cold
        
        .append(OpCode::PUSH4)
        .append("aaaaaaaa")
        .append(OpCode::EXTCODESIZE) // warm

        .append(OpCode::PUSH4)
        .append("ffffffff")
        .append(OpCode::EXTCODESIZE) // warm

        .append(OpCode::PUSH4)
        .append("eeeeeeee")
        .append(OpCode::EXTCODESIZE) // cold
        .clone();
    
    let expected = 21000
        + 2400*3    // account access * 3
        + 1900      // storage access * 1
        + 3 * 5     // push * 5
        + 100       // warm sload
        + 2100      // cold sload
        + 100       // warm extcodesize
        + 100       // warm extcodesize
        + 2600      // cold extcodesize
        ;
    
    let mut tester = EvmEmulator::new_with(get_default_context());
    let result = tester
        .with_to(address_default())
        .with_default_gas()
        .add_accessed_account(address_default())
        .add_accessed_account(address_default())  // dupulicate costs
        .add_accessed_account(address_ext())
        .add_accessed_storage(address_default(), U256::zero())
        .enable_execution_cost()
        .run_code_as(code, Revision::Berlin);
    
    result.expect_status(StatusCode::Success)
        .expect_output("")
        .expect_gas(expected);
}

#[test]
fn test_eip2930_sstore() {
    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append(2)  // new value
        .append(OpCode::PUSH1)
        .append(0)  // warm key
        .append(OpCode::SSTORE)  // warm

        .append(OpCode::PUSH1)
        .append(1)  // new value
        .append(OpCode::PUSH1)
        .append(1)  // cold key
        .append(OpCode::SSTORE)  // cold
        .clone();

    let expected = 21000
        + 1900      // storage access * 1
        + 2400      // account(to) access * 1
        + 3 * 4     // push * 2
        + 2900      // warm sstore
        + 22100     // cold sstore
        ;

    let mut tester = EvmEmulator::new_with(get_default_context());
    let result = tester
        .with_to(address_default())
        .with_default_gas()
        .with_storage(address_default(), U256::zero(), U256::from(0x01))
        .add_accessed_storage(address_default(), U256::zero())
        .enable_execution_cost()
        .run_code_as(code, Revision::Berlin);
    
    result.expect_status(StatusCode::Success)
        .expect_output("")
        .expect_gas(expected);
}