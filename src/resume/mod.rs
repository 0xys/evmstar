use ethereum_types::U256;

pub enum Resume {
    Init,
    Balance(U256),
}