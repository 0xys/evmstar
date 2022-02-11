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

fn consumed_gas(amount: i64) -> i64 {
    i64::max_value() - amount
}

#[test]
pub fn test_pc() {
    let host = TransientHost::new();
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PC)
        .append_opcode(OpCode::POP)
        .append_opcode(OpCode::PUSH3)
        .append(&[0x00, 0x00, 0x00])
        .append_opcode(OpCode::POP)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::POP)
        .append_opcode(OpCode::PC)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("000000000000000000000000000000000000000000000000000000000000000a").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(31), output.gas_left);
}