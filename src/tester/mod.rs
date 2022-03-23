use std::{rc::Rc, cell::RefCell};

use bytes::Bytes;
use ethereum_types::{Address, U256};
use hex::decode;

use crate::{
    model::{code::Code, evmc::{Output, StatusCode, TxContext}, revision::Revision},
    executor::{callstack::CallScope, executor::Executor},
    host::stateful::StatefulHost
};

#[derive(Clone)]
pub struct EvmTester {
    scope: CallScope,
    host: Rc<RefCell<StatefulHost>>
}

pub struct EvmResult {
    host: Rc<RefCell<StatefulHost>>,
    scope: CallScope,
    output: Output
}

impl EvmResult {
    pub fn expect_output<'a>(&'a self, hex: &str) -> &'a Self {
        let data = decode(hex).unwrap();
        assert_eq!(Bytes::from(data), self.output.data);
        self
    }
    pub fn expect_gas<'a>(&'a self, gas: i64) -> &'a Self {
        assert_eq!(gas, self.scope.gas_limit - self.output.gas_left);
        self
    }
    pub fn expect_gas_refund<'a>(&'a self, gas_refund: i64) -> &'a Self {
        assert_eq!(gas_refund, self.output.gas_refund);
        self
    }
    pub fn expect_status<'a>(&'a self, status_code: StatusCode) -> &'a Self {
        assert_eq!(status_code, self.output.status_code);
        self
    }
    pub fn expect_storage<'a>(&'a self, address: Address, key: U256, expected_value: U256) -> &'a Self {
        let value = (*self.host).borrow().debug_get_storage(address, key);
        assert_eq!(expected_value, value);
        self
    }
}

impl EvmTester {
    pub fn new() -> Self {
        let host = StatefulHost::new();
        let host = Rc::new(RefCell::new(host));
        EvmTester{
            scope: CallScope::default(),
            host
        }
    }

    pub fn new_with(context: TxContext) -> Self {
        let host = StatefulHost::new_with(context);
        let host = Rc::new(RefCell::new(host));
        EvmTester{
            scope: CallScope::default(),
            host
        }
    }

    pub fn with_scope<'a>(&'a mut self, scope: CallScope) -> &'a mut Self {
        self.scope = scope;
        self
    }
    pub fn with_to<'a>(&'a mut self, to: Address) -> &'a mut Self {
        self.scope.to = to;
        self
    }
    pub fn with_code<'a>(&'a mut self, code: Code) -> &'a mut Self {
        self.scope.code = code;
        self
    }
    pub fn with_gas_limit<'a>(&'a mut self, gas_limit: i64) -> &'a mut Self {
        self.scope.gas_limit = gas_limit;
        self
    }
    pub fn with_gas_left<'a>(&'a mut self, gas_left: i64) -> &'a mut Self {
        self.scope.gas_left = gas_left;
        self
    }

    pub fn with_storage<'a>(&'a mut self, address: Address, key: U256, value: U256) -> &'a mut Self {
        (*self.host).borrow_mut().debug_set_storage(address, key, value);
        self
    }

    pub fn with_storage_always_warm<'a>(&'a mut self) -> &'a mut Self {
        (*self.host).borrow_mut().debug_set_storage_as_warm();
        self
    }

    pub fn with_contract_deployed<'a>(&'a mut self, contract_address: &str, code: Code, balance: U256) -> &'a mut Self {
        (*self.host).borrow_mut().debug_deploy_contract(contract_address, code, balance);
        self
    }

    pub fn run(&mut self) -> EvmResult {
        let mut executor = Executor::new_with_tracing(self.host.clone());
        
        let output = executor.execute_raw_with(self.scope.clone());
        EvmResult{
            host: self.host.clone(),
            scope: self.scope.clone(),
            output
        }
    }

    pub fn run_as(&mut self, revision: Revision) -> EvmResult {
        let mut executor = Executor::new_with_tracing(self.host.clone());
        executor.set_revision(revision);

        let output = executor.execute_raw_with(self.scope.clone());
        EvmResult{
            host: self.host.clone(),
            scope: self.scope.clone(),
            output
        }
    }

    pub fn run_code(&mut self, code: Code) -> EvmResult {
        self.scope.code = code;
        let mut executor = Executor::new_with_tracing(self.host.clone());

        let output = executor.execute_raw_with(self.scope.clone());
        EvmResult{
            host: self.host.clone(),
            scope: self.scope.clone(),
            output
        }
    }

    pub fn run_code_as(&mut self, code: Code, revision: Revision) -> EvmResult {
        self.scope.code = code;
        let mut executor = Executor::new_with_tracing(self.host.clone());
        executor.set_revision(revision);

        let output = executor.execute_raw_with(self.scope.clone());
        EvmResult{
            host: self.host.clone(),
            scope: self.scope.clone(),
            output
        }
    }
}