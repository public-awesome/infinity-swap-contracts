use crate::{
    helpers::constants::{MIN_PRICE, POOL_CREATION_FEE, TRADING_FEE_BPS},
    setup::setup_marketplace::setup_marketplace,
};

use anyhow::Error;
use cosmwasm_std::{Addr, Uint128};
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

pub fn setup_infinity_global(
    router: &mut StargazeApp,
    creator: &Addr,
    infinity_index: &Addr,
    infinity_factory: &Addr,
) -> Result<Addr, anyhow::Error> {
    let infinity_global_code_id = router.store_code(contract_infinity_global());
    let msg = infinity_global::msg::InstantiateMsg {
        infinity_index: infinity_index.to_string(),
        infinity_factory: infinity_factory.to_string(),
        min_price: Uint128::from(MIN_PRICE),
        pool_creation_fee: Uint128::from(POOL_CREATION_FEE),
        trading_fee_bps: TRADING_FEE_BPS,
    };
    router.instantiate_contract(
        infinity_global_code_id,
        creator.clone(),
        &msg,
        &[],
        "InfinityIndex",
        None,
    )
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
    infinity_global: &Addr,
) -> Result<Addr, anyhow::Error> {
    let infinity_index_code_id = router.store_code(contract_infinity_index());
    let msg = infinity_index::msg::InstantiateMsg {
        infinity_global: infinity_global.to_string(),
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
    pub infinity_global: Addr,
    pub infinity_index: Addr,
    pub infinity_factory: Addr,
    pub infinity_pool_code_id: u64,
}

pub fn setup_infinity_test(num_tokens: u32) -> Result<InfinityTestSetup, Error> {
    let mut vt = standard_minter_template(num_tokens);

    let marketplace = setup_marketplace(&mut vt.router, vt.accts.creator.clone()).unwrap();
    setup_block_time(&mut vt.router, GENESIS_MINT_START_TIME, None);

    let pre_infinity_index = Addr::unchecked("contract5");
    let infinity_factory = Addr::unchecked("contract6");

    let infinity_global = setup_infinity_global(
        &mut vt.router,
        &vt.accts.creator,
        &pre_infinity_index,
        &infinity_factory,
    )?;

    let infinity_index = setup_infinity_index(&mut vt.router, &vt.accts.creator, &infinity_global)?;

    assert_eq!(pre_infinity_index, infinity_index);

    let infinity_pool_code_id = vt.router.store_code(contract_infinity_pool());

    Ok(InfinityTestSetup {
        vending_template: vt,
        marketplace,
        infinity_global,
        infinity_index,
        infinity_factory,
        infinity_pool_code_id,
    })
}
