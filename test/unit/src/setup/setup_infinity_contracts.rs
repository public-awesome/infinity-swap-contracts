use cosmwasm_std::Addr;
use cw_multi_test::{Contract, ContractWrapper, Executor};
use sg_multi_test::StargazeApp;
use sg_std::StargazeMsgWrapper;

pub fn contract_infinity_pool() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        infinity_pool::execute::execute,
        infinity_pool::instantiate::instantiate,
        infinity_pool::query::query,
    );
    Box::new(contract)
}

pub fn contract_infinity_index() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        infinity_index::execute::execute,
        infinity_index::instantiate::instantiate,
        infinity_index::query::query,
    );
    Box::new(contract)
}

pub fn setup_infinity_index(
    router: &mut StargazeApp,
    creator: &Addr,
    marketplace: &Addr,
    infinity_factory: &Addr,
) -> Result<Addr, anyhow::Error> {
    let infinity_index_code_id = router.store_code(contract_infinity_index());
    let msg = infinity_index::msg::InstantiateMsg {
        global_gov: marketplace.to_string(),
        infinity_factory: infinity_factory.to_string(),
    };
    router.instantiate_contract(
        infinity_index_code_id,
        creator.clone(),
        &msg,
        &[],
        "InfinityIndex",
        None,
    )
}

pub fn contract_infinity_router() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        infinity_router::execute::execute,
        infinity_router::instantiate::instantiate,
        infinity_router::query::query,
    );
    Box::new(contract)
}

pub fn setup_infinity_router(
    router: &mut StargazeApp,
    creator: &Addr,
    marketplace: &Addr,
    infinity_index: &Addr,
) -> Result<Addr, anyhow::Error> {
    let infinity_router_code_id = router.store_code(contract_infinity_router());
    let msg = infinity_router::msg::InstantiateMsg {
        infinity_index: infinity_index.to_string(),
    };
    router.instantiate_contract(
        infinity_router_code_id,
        creator.clone(),
        &msg,
        &[],
        "InfinityRouter",
        None,
    )
}
