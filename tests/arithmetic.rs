use std::{rc::Rc, cell::RefCell};

use bytes::Bytes;

use evmstar::host::host::TransientHost;
use evmstar::executor::executor::Executor;
#[allow(unused_imports)]
use evmstar::model::{
    code::{
        Code, Append,
    },
    opcode::OpCode,
    evmc::{
        StatusCode, FailureKind,
    },
};

use hex::{decode};

fn consumed_gas(amount: i64) -> i64 {
    i64::max_value() - amount
}

#[test]
pub fn test_add() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new(host.clone());
    let mut builder = Code::builder();

    let code = builder
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
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("0000000000000000000000000000000000000000000000000000000000000005").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(24), output.gas_left);
}

#[test]
pub fn test_add_overflow() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new(host.clone());
    let mut builder = Code::builder();

    let u256_max = decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

    let code = builder
        .append(OpCode::PUSH32)
        .append(u256_max.as_slice())
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
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(24), output.gas_left);
}

#[test]
pub fn test_sub() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new(host.clone());
    let mut builder = Code::builder();

    let code = builder
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
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("0000000000000000000000000000000000000000000000000000000000000002").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(24), output.gas_left);
}

#[test]
pub fn test_sub_underflow() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new(host.clone());
    let mut builder = Code::builder();

    let code = builder
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
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(24), output.gas_left);
}

#[test]
pub fn test_mul() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new(host.clone());
    let mut builder = Code::builder();

    let code = builder
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
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("0000000000000000000000000000000000000000000000000000000000000008").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(26), output.gas_left);
}

#[test]
pub fn test_mul_overflow() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new(host.clone());
    let mut builder = Code::builder();

    let u256_max = decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

    let code = builder
        .append(OpCode::PUSH32)
        .append(u256_max.as_slice())
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
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(26), output.gas_left);
}


#[test]
pub fn test_div() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new(host.clone());
    let mut builder = Code::builder();

    let u256_max = decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

    let code = builder
        .append(OpCode::PUSH1)
        .append("03")
        .append(OpCode::PUSH32)
        .append(u256_max.as_slice())
        .append(OpCode::DIV)
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("5555555555555555555555555555555555555555555555555555555555555555").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(26), output.gas_left);
}

#[test]
pub fn test_sdiv() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new(host.clone());
    let mut builder = Code::builder();

    let bn = decode("0000000000000000000000ffffffffffffffffffffffffffffffffffffffffff").unwrap();
    let code = builder
        .append(OpCode::PUSH1)
        .append("03")
        .append(OpCode::PUSH32)
        .append(bn.as_slice())
        .append(OpCode::DIV)
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("0000000000000000000000555555555555555555555555555555555555555555").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(26), output.gas_left);
}

#[test]
fn test_arith() {   // https://github.com/ethereum/tests/blob/develop/src/GeneralStateTestsFiller/VMTests/vmArithmeticTest/arithFiller.yml
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new(host.clone());
    let mut builder = Code::builder();

    let code = builder
        .append("600160019001600702600501600290046004906021900560170160030260059007600303600960110A60005260206000F3");
    
    let output = executor.execute_raw(&code);

    let data = decode("0000000000000000000000000000000000000000000000000000001b9c636491").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(166), output.gas_left);
}

#[test]
fn test_comparison() {    // from evmordin tests
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new(host.clone());
    let mut builder = Code::builder();

    let code = builder
        .append("60006001808203808001")  // 0 1 -1 -2
        .append("828210600053")          // m[0] = -1 < 1
        .append("828211600153")          // m[1] = -1 > 1
        .append("828212600253")          // m[2] = -1 s< 1
        .append("828213600353")          // m[3] = -1 s> 1
        .append("828214600453")          // m[4] = -1 == 1
        .append("818112600553")          // m[5] = -2 s< -1
        .append("818113600653")          // m[6] = -2 s> -1
        .append("60076000f3");
    
    let output = executor.execute_raw(&code);

    let data = decode("00010100000100").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(138), output.gas_left);
}

#[test]
fn test_bitwise() {    // from evmordin tests
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new(host.clone());
    let mut builder = Code::builder();

    let code = builder
        .append("60aa60ff")       // aa ff
        .append("818116600053")   // m[0] = aa & ff
        .append("818117600153")   // m[1] = aa | ff
        .append("818118600253")   // m[2] = aa ^ ff
        .append("60036000f3");
    
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(vec![0xaa & 0xff, 0xaa | 0xff, 0xaa ^ 0xff]), output.data);
    assert_eq!(consumed_gas(60), output.gas_left);
}