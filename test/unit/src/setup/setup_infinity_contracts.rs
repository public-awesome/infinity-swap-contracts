use crate::setup::setup_marketplace::setup_marketplace;
use anyhow::Error;
use cosmwasm_std::Addr;
use cw_multi_test::{Contract, ContractWrapper, Executor};
use sg_multi_test::StargazeApp;
use sg_std::{StargazeMsgWrapper, GENESIS_MINT_START_TIME};
use test_suite::common_setup::{
    msg::MinterTemplateResponse, setup_accounts_and_block::setup_block_time,
};

use super::{msg::MarketAccounts, templates::standard_minter_template};

pub fn contract_infinity_global() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        infinity_global::execute::execute,
        infinity_global::instantiate::instantiate,
        infinity_global::query::query,
    )
    .with_sudo(infinity_global::sudo::sudo);
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
) -> Result<Addr, anyhow::Error> {
    let infinity_index_code_id = router.store_code(contract_infinity_index());
    let msg = infinity_index::msg::InstantiateMsg {
        global_gov: marketplace.to_string(),
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

pub struct InfinityTestSetup {
    pub vending_template: MinterTemplateResponse<MarketAccounts>,
    pub marketplace: Addr,
    pub infinity_index: Addr,
    pub infinity_router: Addr,
    pub infinity_pool_code_id: u64,
}

pub fn setup_infinity_test(num_tokens: u32) -> Result<InfinityTestSetup, Error> {
    let mut vt = standard_minter_template(num_tokens);

    let marketplace = setup_marketplace(&mut vt.router, vt.accts.creator.clone()).unwrap();
    setup_block_time(&mut vt.router, GENESIS_MINT_START_TIME, None);

    let infinity_pool_code_id = vt.router.store_code(contract_infinity_pool());

    let infinity_factory = Addr::unchecked("infinity_factory");

    let infinity_index = setup_infinity_index(&mut vt.router, &vt.accts.creator, &marketplace)?;
    let infinity_router =
        setup_infinity_router(&mut vt.router, &vt.accts.creator, &marketplace, &infinity_index)?;

    Ok(InfinityTestSetup {
        vending_template: vt,
        marketplace,
        infinity_pool_code_id,
        infinity_index,
        infinity_router,
    })
}
