use bytes::Bytes;
use evmstar::model::evmc::TxContext;
use evmstar::emulator::EvmEmulator;
use evmstar::model::{
    code::{
        Code, Append,
    },
    opcode::OpCode,
    evmc::{
        StatusCode,
    },
};

#[test]
pub fn test_add() {
    let mut emulator = EvmEmulator::new_transient_with(TxContext::default());
    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append("02")
        .append(OpCode::PUSH1)
        .append("03")
        .append(OpCode::ADD)
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN)
        .clone();
    
    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Success)
        .expect_gas(24)
        .expect_output("0000000000000000000000000000000000000000000000000000000000000005");
}

#[test]
pub fn test_add_overflow() {
    let mut emulator = EvmEmulator::new_transient_with(TxContext::default());
    let code = Code::builder()
        .append(OpCode::PUSH32)
        .append("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
        .append(OpCode::PUSH1)
        .append("01")
        .append(OpCode::ADD)
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN)
        .clone();
    
    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Success)
        .expect_gas(24)
        .expect_output("0000000000000000000000000000000000000000000000000000000000000000");
}

#[test]
pub fn test_sub() {
    let mut emulator = EvmEmulator::new_transient_with(TxContext::default());
    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append("02")
        .append(OpCode::PUSH1)
        .append("04")
        .append(OpCode::SUB)     // 4 - 2
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN)
        .clone();
    
    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Success)
        .expect_gas(24)
        .expect_output("0000000000000000000000000000000000000000000000000000000000000002");
}

#[test]
pub fn test_sub_underflow() {
    let mut emulator = EvmEmulator::new_transient_with(TxContext::default());

    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append("01")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::SUB)     // 0 - 1 = underflow
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN)
        .clone();
    
    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Success)
        .expect_gas(24)
        .expect_output("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
}

#[test]
pub fn test_mul() {
    let mut emulator = EvmEmulator::new_transient_with(TxContext::default());

    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append("04")
        .append(OpCode::PUSH1)
        .append("02")
        .append(OpCode::MUL)
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN)
        .clone();

    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Success)
        .expect_gas(26)
        .expect_output("0000000000000000000000000000000000000000000000000000000000000008");
}

#[test]
pub fn test_mul_overflow() {
    let mut emulator = EvmEmulator::new_transient_with(TxContext::default());

    let code = Code::builder()
        .append(OpCode::PUSH32)
        .append("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
        .append(OpCode::PUSH1)
        .append("02")
        .append(OpCode::MUL)
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN)
        .clone();
 
    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Success)
        .expect_gas(26)
        .expect_output("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe");
}


#[test]
pub fn test_div() {
    let mut emulator = EvmEmulator::new_transient_with(TxContext::default());

    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append("03")
        .append(OpCode::PUSH32)
        .append("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
        .append(OpCode::DIV)
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN)
        .clone();
    
    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Success)
        .expect_gas(26)
        .expect_output("5555555555555555555555555555555555555555555555555555555555555555");
}

#[test]
pub fn test_sdiv() {
    let mut emulator = EvmEmulator::new_transient_with(TxContext::default());

    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append("03")
        .append(OpCode::PUSH32)
        .append("0000000000000000000000ffffffffffffffffffffffffffffffffffffffffff")
        .append(OpCode::DIV)
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN)
        .clone();
    
    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Success)
        .expect_gas(26)
        .expect_output("0000000000000000000000555555555555555555555555555555555555555555");
}

#[test]
fn test_arith() {   // https://github.com/ethereum/tests/blob/develop/src/GeneralStateTestsFiller/VMTests/vmArithmeticTest/arithFiller.yml
    let mut emulator = EvmEmulator::new_transient_with(TxContext::default());

    let code = Code::builder()
        .append("600160019001600702600501600290046004906021900560170160030260059007600303600960110A60005260206000F3")
        .clone();

    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Success)
        .expect_gas(166)
        .expect_output("0000000000000000000000000000000000000000000000000000001b9c636491");
}

#[test]
fn test_comparison() {    // from evmordin tests
    let mut emulator = EvmEmulator::new_transient_with(TxContext::default());

    let code = Code::builder()
        .append("60006001808203808001")  // 0 1 -1 -2
        .append("828210600053")          // m[0] = -1 < 1
        .append("828211600153")          // m[1] = -1 > 1
        .append("828212600253")          // m[2] = -1 s< 1
        .append("828213600353")          // m[3] = -1 s> 1
        .append("828214600453")          // m[4] = -1 == 1
        .append("818112600553")          // m[5] = -2 s< -1
        .append("818113600653")          // m[6] = -2 s> -1
        .append("60076000f3")
        .clone();
    
    let result = emulator.run_code(code);
    
    result.expect_status(StatusCode::Success)
        .expect_gas(138)
        .expect_output("00010100000100");
}

#[test]
fn test_bitwise() {    // from evmordin tests
    let mut emulator = EvmEmulator::new_transient_with(TxContext::default());

    let code = Code::builder()
        .append("60aa60ff")       // aa ff
        .append("818116600053")   // m[0] = aa & ff
        .append("818117600153")   // m[1] = aa | ff
        .append("818118600253")   // m[2] = aa ^ ff
        .append("60036000f3")
        .clone();

    let result = emulator.run_code(code);
    
    result.expect_status(StatusCode::Success)
        .expect_gas(60);

    assert_eq!(Bytes::from(vec![0xaa & 0xff, 0xaa | 0xff, 0xaa ^ 0xff]), result.output.data);
}