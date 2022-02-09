use bytes::Bytes;

use evmstar::executor::callstack::{CallStack, CallContext};
use evmstar::interpreter::{
    Interrupt,
    interpreter::Interpreter
};
use evmstar::host::host::TransientHost;
use evmstar::interpreter::stack::{Stack};
use evmstar::executor::executor::Executor;
use evmstar::model::{
    code::{Code},
    opcode::OpCode,
    evmc::{
        StatusCode, FailureKind,
    },
};

use hex::encode;

#[test]
pub fn test_stack() {

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
    
        println!("code: {:?}", encode(&code.0));

    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(vec![0xff]), output.data);
}