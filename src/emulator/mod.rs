use std::{rc::Rc, cell::RefCell};

use bytes::Bytes;
use ethereum_types::{Address, U256};
use hex::decode;

use crate::{
    model::{code::Code, evmc::{Output, StatusCode, TxContext, AccessList}, revision::Revision},
    executor::{callstack::CallScope, executor::Executor},
    host::{stateful::StatefulHost, Host, transient::TransientHost}
};

pub struct EvmEmulator {
    scope: CallScope,
    host: Rc<RefCell<dyn Host>>,

    is_execution_cost_enabled: bool,
    access_list: AccessList,
}

pub struct EvmResult {
    host: Rc<RefCell<dyn Host>>,
    scope: CallScope,
    pub output: Output,
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

impl EvmEmulator {
    pub fn new_transient_with(context: TxContext) -> Self {
        let host = TransientHost::new_with(context);
        let host = Rc::new(RefCell::new(host));
        EvmEmulator{
            scope: CallScope::default(),
            host,
            is_execution_cost_enabled: false,
            access_list: AccessList::default(),
        }
    }

    pub fn new_stateful_with(context: TxContext) -> Self {
        let host = StatefulHost::new_with(context);
        let host = Rc::new(RefCell::new(host));
        EvmEmulator{
            scope: CallScope::default(),
            host,
            is_execution_cost_enabled: false,
            access_list: AccessList::default(),
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
    pub fn with_default_gas<'a>(&'a mut self) -> &'a mut Self {
        self.scope.gas_limit = i32::max_value() as i64;
        self.scope.gas_left = i32::max_value() as i64;
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
    pub fn with_contract_deployed2<'a>(&'a mut self, contract_address2: Address, code: Code, balance: U256) -> &'a mut Self {
        (*self.host).borrow_mut().debug_deploy_contract2(contract_address2, code, balance);
        self
    }

    pub fn enable_execution_cost<'a>(&'a mut self) -> &'a mut Self {
        self.is_execution_cost_enabled = true;
        self
    }

    pub fn add_accessed_account<'a>(&'a mut self, address: Address) -> &'a mut Self {
        self.access_list.add_account(address);
        self
    }
    pub fn add_accessed_storage<'a>(&'a mut self, address: Address, key: U256) -> &'a mut Self {
        self.access_list.add_storage(address, key);
        self
    }

    pub fn run(&mut self) -> EvmResult {
        self.run_as(Revision::Shanghai)
    }

    pub fn run_as(&mut self, revision: Revision) -> EvmResult {
        let mut executor = 
            if !self.is_execution_cost_enabled {
                Executor::new_with_tracing(self.host.clone())
            } else {
                Executor::new_with_execution_cost(self.host.clone(), true, revision)
            };
        executor.set_revision(revision);

        let output = 
            if self.access_list.is_empty() {
                executor.execute_raw_with(self.scope.clone())
            } else {
                executor.execute_with_access_list(self.scope.clone(), self.access_list.clone())
            };
        EvmResult{
            host: self.host.clone(),
            scope: self.scope.clone(),
            output
        }
    }

    pub fn run_code(&mut self, code: Code) -> EvmResult {
        self.scope.code = code;
        self.run()
    }

    pub fn run_code_as(&mut self, code: Code, revision: Revision) -> EvmResult {
        self.scope.code = code;
        self.run_as(revision)
    }
}