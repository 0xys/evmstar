# EVM star

# Toy Example
```rs
let host = TransientHost::new();
let mut executor = Executor::new(Box::new(host));
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
```
