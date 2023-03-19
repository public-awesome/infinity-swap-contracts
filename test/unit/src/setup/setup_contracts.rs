use cw_multi_test::{Contract, ContractWrapper};
use sg_std::StargazeMsgWrapper;

pub fn contract_marketplace() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg_marketplace::execute::execute,
        sg_marketplace::execute::instantiate,
        sg_marketplace::query::query,
    )
    .with_sudo(sg_marketplace::sudo::sudo)
    .with_reply(sg_marketplace::execute::reply)
    .with_migrate(sg_marketplace::execute::migrate);
    Box::new(contract)
}

pub fn contract_infinity_pool() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        infinity_pool::execute::execute,
        infinity_pool::instantiate::instantiate,
        infinity_pool::query::query,
    );
    Box::new(contract)
}
