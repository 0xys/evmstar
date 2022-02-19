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
pub fn test_push() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(0xff)
        .append_opcode(OpCode::PUSH1)
        .append(0x00)
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(0x01)
        .append_opcode(OpCode::PUSH1)
        .append(31)
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(vec![0xff]), output.data);
    assert_eq!(consumed_gas(18), output.gas_left);
}

#[test]
pub fn test_pop_empty() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::POP);
    
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Failure(FailureKind::StackUnderflow), output.status_code);
}

#[test]
pub fn test_push_overflow() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let mut code: Vec<u8> = vec![];
    for _ in 0..1025 {  // push 1025 items into stack.
        code.push(OpCode::PUSH1.to_u8());
        code.push(0x00);
    }
    code.push(OpCode::RETURN.to_u8());

    let code = builder.append(code.as_slice());
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Failure(FailureKind::StackOverflow), output.status_code);
}

#[test]
pub fn test_push2() {
    {
        let host = TransientHost::new();
        let mut executor = Executor::new(Box::new(host));
        let mut builder = Code::builder();
    
        let code = builder
            .append_opcode(OpCode::PUSH2)
            .append("1122")
            .append_opcode(OpCode::PUSH1)
            .append("00")
            .append_opcode(OpCode::MSTORE)
            .append_opcode(OpCode::PUSH1)
            .append("02")
            .append_opcode(OpCode::PUSH1)
            .append(30)
            .append_opcode(OpCode::RETURN);
        
        let output = executor.execute_raw(&code);
    
        assert_eq!(StatusCode::Success, output.status_code);
        assert_eq!(Bytes::from(vec![0x11, 0x22]), output.data);
        assert_eq!(consumed_gas(18), output.gas_left);
    }
    {
        let host = TransientHost::new();
        let mut executor = Executor::new(Box::new(host));
        let mut builder = Code::builder();
    
        let code = builder
            .append_opcode(OpCode::PUSH2)
            .append("1122")
            .append_opcode(OpCode::PUSH1)
            .append("00")
            .append_opcode(OpCode::MSTORE)
            .append_opcode(OpCode::PUSH1)
            .append("02")
            .append_opcode(OpCode::PUSH1)
            .append(31)      // cause memory expansion
            .append_opcode(OpCode::RETURN);
        
        let output = executor.execute_raw(&code);
    
        assert_eq!(StatusCode::Success, output.status_code);
        assert_eq!(Bytes::from(vec![0x22, 0x00]), output.data);
        assert_eq!(consumed_gas(21), output.gas_left);
    }
}

#[test]
pub fn test_push32() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let data = decode("ff00000000000000000000000000000000000000000000000000000011223344").unwrap();

    let code = builder
        .append_opcode(OpCode::PUSH32)
        .append(data.as_slice())
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append("20")
        .append_opcode(OpCode::PUSH1)
        .append(0)
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(18), output.gas_left);
}

#[test]
pub fn test_push32_with_expansion() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let data =          decode("ff000000000000000000000000000000ff000000000000000000000011223344").unwrap();
    let data_right =    decode("ff00000000000000000000001122334400000000000000000000000000000000").unwrap();

    let code = builder
        .append_opcode(OpCode::PUSH32)
        .append(data.as_slice())
        .append_opcode(OpCode::PUSH1)
        .append("10")
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append("20")
        .append_opcode(OpCode::PUSH1)
        .append("20")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data_right), output.data);
    assert_eq!(consumed_gas(21), output.gas_left);
}

#[test]
pub fn test_dup1() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append("01")
        .append_opcode(OpCode::DUP1)
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append("40")
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let memory = decode("00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(memory), output.data);
    assert_eq!(consumed_gas(21), output.gas_left);
}

#[test]
pub fn test_dup2() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append("01")
        .append_opcode(OpCode::PUSH1)
        .append("01")
        .append_opcode(OpCode::DUP2)
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append("40")
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let memory = decode("00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(memory), output.data);
    assert_eq!(consumed_gas(24), output.gas_left);
}

#[test]
pub fn test_dup3() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append("01")
        .append_opcode(OpCode::PUSH1)
        .append("ff")
        .append_opcode(OpCode::PUSH1)
        .append("01")
        .append_opcode(OpCode::DUP3)
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append("40")
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let memory = decode("00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(memory), output.data);
    assert_eq!(consumed_gas(27), output.gas_left);
}

#[test]
pub fn test_dup_overflow() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append("01")
        .append_opcode(OpCode::DUP16)   // overflow
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append("40")
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    assert_eq!(StatusCode::Failure(FailureKind::StackOverflow), output.status_code);
}

#[test]
pub fn test_swap2swap3() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append("02")
        .append_opcode(OpCode::PUSH1)
        .append("03")
        .append_opcode(OpCode::PUSH1)
        .append("04")
        .append_opcode(OpCode::PUSH1)
        .append("05")
        .append_opcode(OpCode::PUSH1)
        .append("06")
        .append_opcode(OpCode::SWAP2)
        .append_opcode(OpCode::SWAP3)
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append("40")
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let memory = decode("00000000000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(memory), output.data);
    assert_eq!(consumed_gas(36), output.gas_left);
}

#[test]
fn test_all() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    /*
PUSH1 0x00
PUSH1 0xaa
PUSH2 0xaaaa
PUSH3 0xaaaaaa
PUSH4 0xaaaaaaaa
PUSH5 0xaaaaaaaaaa
PUSH6 0xaaaaaaaaaaaa
PUSH7 0xaaaaaaaaaaaaaa
PUSH8 0xaaaaaaaaaaaaaaaa
PUSH9  0xaaaaaaaaaaaaaaaaaa
PUSH10 0xaaaaaaaaaaaaaaaaaaaa
PUSH11 0xaaaaaaaaaaaaaaaaaaaaaa
PUSH12 0xaaaaaaaaaaaaaaaaaaaaaaaa
PUSH13 0xaaaaaaaaaaaaaaaaaaaaaaaaaa
PUSH14 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaa
PUSH15 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
PUSH16 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
SWAP1
SWAP2
SWAP3
SWAP4
SWAP5
SWAP6
SWAP7
SWAP8
SWAP9
SWAP10
SWAP11
SWAP12
SWAP13
SWAP14
SWAP15
SWAP16
DUP16
DUP15
DUP14
DUP13
DUP12
DUP11
DUP10
DUP9
DUP8
DUP7
DUP6
DUP5
DUP4
DUP3
DUP2
DUP1
PUSH1 0x00
MSTORE
PUSH1 0x20
MSTORE
PUSH1 0x40
MSTORE
PUSH1 0x60
MSTORE
PUSH1 0x80
MSTORE
PUSH1 0xa0
MSTORE
PUSH1 0xc0
MSTORE
PUSH1 0xe0
MSTORE
PUSH2 0x0100
MSTORE
PUSH2 0x0120
MSTORE
PUSH2 0x0140
MSTORE
PUSH2 0x0160
MSTORE
PUSH2 0x0180
MSTORE
PUSH2 0x01a0
MSTORE
PUSH2 0x01c0
MSTORE
PUSH2 0x01e0
MSTORE
PUSH2 0x0200
MSTORE
PUSH2 0x0220
MSTORE
PUSH2 0x0240
MSTORE
PUSH2 0x0260
MSTORE
PUSH2 0x0280
MSTORE
PUSH2 0x02a0
MSTORE
PUSH2 0x02c0
MSTORE
PUSH2 0x02e0
MSTORE
PUSH2 0x0300
MSTORE
PUSH2 0x0320
MSTORE
PUSH2 0x0340
MSTORE
PUSH2 0x0360
MSTORE
PUSH2 0x0380
MSTORE
PUSH2 0x03a0
MSTORE
PUSH2 0x03c0
MSTORE
PUSH2 0x03e0
MSTORE
PUSH2 0x0400
PUSH1 0x00
RETURN
    */
    let code = builder
        .append("600060aa61aaaa62aaaaaa63aaaaaaaa64aaaaaaaaaa65aaaaaaaaaaaa66aaaaaaaaaaaaaa67aaaaaaaaaaaaaaaa68aaaaaaaaaaaaaaaaaa69aaaaaaaaaaaaaaaaaaaa6aaaaaaaaaaaaaaaaaaaaaaa6baaaaaaaaaaaaaaaaaaaaaaaa6caaaaaaaaaaaaaaaaaaaaaaaaaa6daaaaaaaaaaaaaaaaaaaaaaaaaaaa6eaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa6faaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa909192939495969798999a9b9c9d9e9f8f8e8d8c8b8a8988878685848382818060005260205260405260605260805260a05260c05260e05261010052610120526101405261016052610180526101a0526101c0526101e05261020052610220526102405261026052610280526102a0526102c0526102e05261030052610320526103405261036052610380526103a0526103c0526103e0526104006000f3");

    let output = executor.execute_raw(&code);
    let memory = decode("000000000000000000000000000000000000000000000000000000000000aaaa000000000000000000000000000000000000000000000000000000000000aaaa00000000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaa000000000000000000000000000000000000000000000000000000000000aaaa000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaa00000000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaa0000000000000000000000000000000000000000000000000000aaaaaaaaaaaa000000000000000000000000000000000000000000000000000000000000aaaa00000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaa0000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaa00000000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaa000000000000000000000000000000000000000000000000aaaaaaaaaaaaaaaa0000000000000000000000000000000000000000000000000000aaaaaaaaaaaa00000000000000000000000000000000000000000000000000000000aaaaaaaa000000000000000000000000000000000000000000000000000000000000aaaa000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaa00000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaa0000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaa000000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaa00000000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaa0000000000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaa000000000000000000000000000000000000000000000000aaaaaaaaaaaaaaaa00000000000000000000000000000000000000000000000000aaaaaaaaaaaaaa0000000000000000000000000000000000000000000000000000aaaaaaaaaaaa000000000000000000000000000000000000000000000000000000aaaaaaaaaa00000000000000000000000000000000000000000000000000000000aaaaaaaa0000000000000000000000000000000000000000000000000000000000aaaaaa000000000000000000000000000000000000000000000000000000000000aaaa").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(memory), output.data);
    assert_eq!(consumed_gas(443), output.gas_left);
}