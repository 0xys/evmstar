use bytes::Bytes;
use hex_literal::hex;

use evmstar::host::host::TransientHost;
use evmstar::executor::executor::Executor;
#[allow(unused_imports)]
use evmstar::model::{
    code::{Code},
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
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0x02])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x03])
        .append_opcode(OpCode::ADD)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("0000000000000000000000000000000000000000000000000000000000000005").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(24), output.gas_left);
}

#[test]
pub fn test_add_overflow() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let u256_max = decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

    let code = builder
        .append_opcode(OpCode::PUSH32)
        .append(u256_max.as_slice())
        .append_opcode(OpCode::PUSH1)
        .append(&[0x01])
        .append_opcode(OpCode::ADD)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(24), output.gas_left);
}

#[test]
pub fn test_sub() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0x02])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x04])
        .append_opcode(OpCode::SUB)     // 4 - 2
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("0000000000000000000000000000000000000000000000000000000000000002").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(24), output.gas_left);
}

#[test]
pub fn test_sub_underflow() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0x01])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::SUB)     // 0 - 1 = underflow
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(24), output.gas_left);
}

#[test]
pub fn test_mul() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0x04])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x02])
        .append_opcode(OpCode::MUL)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("0000000000000000000000000000000000000000000000000000000000000008").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(26), output.gas_left);
}

#[test]
pub fn test_mul_overflow() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let u256_max = decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

    let code = builder
        .append_opcode(OpCode::PUSH32)
        .append(u256_max.as_slice())
        .append_opcode(OpCode::PUSH1)
        .append(&[0x02])
        .append_opcode(OpCode::MUL)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(26), output.gas_left);
}


#[test]
pub fn test_div() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let u256_max = decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0x03])
        .append_opcode(OpCode::PUSH32)
        .append(u256_max.as_slice())
        .append_opcode(OpCode::DIV)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("5555555555555555555555555555555555555555555555555555555555555555").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(26), output.gas_left);
}

#[test]
pub fn test_sdiv() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let bn = decode("0000000000000000000000ffffffffffffffffffffffffffffffffffffffffff").unwrap();
    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0x03])
        .append_opcode(OpCode::PUSH32)
        .append(bn.as_slice())
        .append_opcode(OpCode::DIV)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let data = decode("0000000000000000000000555555555555555555555555555555555555555555").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(26), output.gas_left);
}

#[test]
fn test_arith() {    // from evmordin tests
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append(&hex!("60116001600003600302"))  // 17 -3
        .append(&hex!("808205"))                // 17 -3 -5
        .append(&hex!("818307"))                // 17 -3 -5 2
        .append(&hex!("910201"))                // 17 17
        .append(&hex!("0315"))                  // 1
        .append(&hex!("60005360016000f3"));
    
    let output = executor.execute_raw(&code);
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(vec![1]), output.data);
    assert_eq!(consumed_gas(74), output.gas_left);
}

#[test]
fn test_comparison() {    // from evmordin tests
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append(&hex!("60006001808203808001"))  // 0 1 -1 -2
        .append(&hex!("828210600053"))          // m[0] = -1 < 1
        .append(&hex!("828211600153"))          // m[1] = -1 > 1
        .append(&hex!("828212600253"))          // m[2] = -1 s< 1
        .append(&hex!("828213600353"))          // m[3] = -1 s> 1
        .append(&hex!("828214600453"))          // m[4] = -1 == 1
        .append(&hex!("818112600553"))          // m[5] = -2 s< -1
        .append(&hex!("818113600653"))          // m[6] = -2 s> -1
        .append(&hex!("60076000f3"));
    
    let output = executor.execute_raw(&code);

    let data = decode("00010100000100").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(138), output.gas_left);
}

#[test]
fn test_bitwise() {    // from evmordin tests
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append(&hex!("60aa60ff"))       // aa ff
        .append(&hex!("818116600053"))   // m[0] = aa & ff
        .append(&hex!("818117600153"))   // m[1] = aa | ff
        .append(&hex!("818118600253"))   // m[2] = aa ^ ff
        .append(&hex!("60036000f3"));
    
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(vec![0xaa & 0xff, 0xaa | 0xff, 0xaa ^ 0xff]), output.data);
    assert_eq!(consumed_gas(60), output.gas_left);
}