use evmstar::host::host::{
    TransientHost,
};
use evmstar::executor::{
    executor::Executor,
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

#[test]
fn test_too_large_code() {
    let mut code: Vec<u8> = vec![];
    for _ in 0..0x6000 {
        code.push(0x00);
    }
    code.push(0x00);
    let mut builder = Code::builder();
    let code = builder.append(code.as_slice());

    for revision in Revision::iter() {
        let host = TransientHost::new();
        let mut executor = Executor::new_with(Box::new(host), false, revision);
        
        let output = executor.execute_raw(&code);
        if revision >= Revision::Spurious {
            assert_eq!(StatusCode::Failure(FailureKind::OutOfGas), output.status_code);
        }else{
            assert_eq!(StatusCode::Success, output.status_code);
        }
    }
}