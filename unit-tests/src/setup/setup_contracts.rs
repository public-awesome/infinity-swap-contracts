use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_multi_test::{Contract, ContractWrapper, Executor};
use cw_utils::Duration;
use sg_marketplace::ExpiryRange;
use sg_multi_test::StargazeApp;
use sg_std::StargazeMsgWrapper;
use stargaze_fair_burn::msg::InstantiateMsg as FairBurnInstantiateMsg;
use stargaze_royalty_registry::{
    msg::InstantiateMsg as RoyaltyRegistryInstantiateMsg, state::Config as RoyaltyRegistryConfig,
};

pub fn contract_fair_burn() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        stargaze_fair_burn::contract::execute,
        stargaze_fair_burn::contract::instantiate,
        stargaze_fair_burn::contract::query,
    );
    Box::new(contract)
}

pub fn setup_fair_burn(router: &mut StargazeApp, creator: &Addr) -> Addr {
    let fair_burn_id = router.store_code(contract_fair_burn());
    router
        .instantiate_contract(
            fair_burn_id,
            creator.clone(),
            &FairBurnInstantiateMsg {
                fee_bps: 5000,
            },
            &[],
            "FairBurn",
            None,
        )
        .unwrap()
}

pub fn contract_royalty_registry() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        stargaze_royalty_registry::execute::execute,
        stargaze_royalty_registry::instantiate::instantiate,
        stargaze_royalty_registry::query::query,
    );
    Box::new(contract)
}

pub fn setup_royalty_registry(router: &mut StargazeApp, creator: &Addr) -> Addr {
    let royalty_registry_id = router.store_code(contract_royalty_registry());
    router
        .instantiate_contract(
            royalty_registry_id,
            creator.clone(),
            &RoyaltyRegistryInstantiateMsg {
                config: RoyaltyRegistryConfig {
                    update_wait_period: 24 * 60 * 60,
                    max_share_delta: Decimal::percent(10),
                },
            },
            &[],
            "FairBurn",
            None,
        )
        .unwrap()
}

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

pub const MIN_EXPIRY: u64 = 24 * 60 * 60; // 24 hours (in seconds)
pub const MAX_EXPIRY: u64 = 180 * 24 * 60 * 60; // 6 months (in seconds)

pub fn setup_marketplace(router: &mut StargazeApp, creator: &Addr) -> Addr {
    let marketplace_id = router.store_code(contract_marketplace());
    let msg = sg_marketplace::msg::InstantiateMsg {
        operators: vec!["operator1".to_string()],
        trading_fee_bps: 200,
        ask_expiry: ExpiryRange::new(MIN_EXPIRY, MAX_EXPIRY),
        bid_expiry: ExpiryRange::new(MIN_EXPIRY, MAX_EXPIRY),
        sale_hook: None,
        max_finders_fee_bps: 1000,
        min_price: Uint128::from(5u128),
        stale_bid_duration: Duration::Time(100),
        bid_removal_reward_bps: 500,
        listing_fee: Uint128::from(100u64),
    };
    router
        .instantiate_contract(marketplace_id, creator.clone(), &msg, &[], "Marketplace", None)
        .unwrap()
}
