use bytes::Bytes;

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

#[test]
pub fn test_push() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0xff])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x01])
        .append_opcode(OpCode::PUSH1)
        .append(&[31])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(vec![0xff]), output.data);
}

#[test]
pub fn test_push2() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH2)
        .append(&[0x11,0x22])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x02])
        .append_opcode(OpCode::PUSH1)
        .append(&[30])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(vec![0x11, 0x22]), output.data);
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
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
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
        .append(&[0x10])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data_right), output.data);
}

#[test]
pub fn test_dup1() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0x01])
        .append_opcode(OpCode::DUP1)
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x40])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let memory = decode("00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(memory), output.data);
}

#[test]
pub fn test_dup2() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0x01])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x01])
        .append_opcode(OpCode::DUP2)
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x40])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let memory = decode("00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(memory), output.data);
}

#[test]
pub fn test_dup3() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0x01])
        .append_opcode(OpCode::PUSH1)
        .append(&[0xff])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x01])
        .append_opcode(OpCode::DUP3)
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x40])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    let memory = decode("00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(memory), output.data);
}

#[test]
pub fn test_dup_overflow() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0x01])
        .append_opcode(OpCode::DUP16)   // overflow
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x40])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let result = match output.status_code {
        StatusCode::Failure(_) => true,
        _ => false
    };
    assert_eq!(true, result);
}

#[test]
pub fn test_swap2swap3() {
    let host = TransientHost::new();
    let mut executor = Executor::new(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)
        .append(&[0x02])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x03])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x04])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x05])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x06])
        .append_opcode(OpCode::SWAP2)
        .append_opcode(OpCode::SWAP3)
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x40])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let memory = decode("00000000000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(memory), output.data);
}