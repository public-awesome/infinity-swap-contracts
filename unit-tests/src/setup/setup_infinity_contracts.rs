use cosmwasm_std::{coin, Addr, Decimal};
use cw_multi_test::{Contract, ContractWrapper, Executor};
use infinity_global::GlobalConfig;
use sg_multi_test::StargazeApp;
use sg_std::{StargazeMsgWrapper, NATIVE_DENOM};

pub const UOSMO: &str = "uosmo";

pub fn contract_infinity_global() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        infinity_global::execute,
        infinity_global::instantiate,
        infinity_global::query,
    )
    .with_sudo(infinity_global::sudo);
    Box::new(contract)
}

#[allow(clippy::too_many_arguments)]
pub fn setup_infinity_global(
    router: &mut StargazeApp,
    creator: String,
    fair_burn: String,
    royalty_registry: String,
    marketplace: String,
    infinity_factory: String,
    infinity_index: String,
    infinity_router: String,
    infinity_pair_code_id: u64,
) -> Addr {
    let infinity_global_code_id = router.store_code(contract_infinity_global());
    let msg = infinity_global::InstantiateMsg {
        global_config: GlobalConfig {
            fair_burn,
            royalty_registry,
            marketplace,
            infinity_factory,
            infinity_index,
            infinity_router,
            infinity_pair_code_id,
            pair_creation_fee: coin(1_000_000, "ustars"),
            fair_burn_fee_percent: Decimal::percent(1),
            default_royalty_fee_percent: Decimal::percent(5),
            max_royalty_fee_percent: Decimal::percent(10),
            max_swap_fee_percent: Decimal::percent(5),
        },
        min_prices: vec![coin(10u128, NATIVE_DENOM), coin(10u128, UOSMO)],
    };
    router
        .instantiate_contract(
            infinity_global_code_id,
            Addr::unchecked(creator),
            &msg,
            &[],
            "Infinity Global",
            None,
        )
        .unwrap()
}

pub fn contract_infinity_factory() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        infinity_factory::execute::execute,
        infinity_factory::instantiate::instantiate,
        infinity_factory::query::query,
    );
    Box::new(contract)
}

pub fn setup_infinity_factory(
    router: &mut StargazeApp,
    creator: &Addr,
    infinity_global: &Addr,
) -> Addr {
    let infinity_factory_code_id = router.store_code(contract_infinity_factory());
    let msg = infinity_factory::msg::InstantiateMsg {
        infinity_global: infinity_global.to_string(),
    };
    router
        .instantiate_contract(
            infinity_factory_code_id,
            creator.clone(),
            &msg,
            &[],
            "Infinity Factory",
            None,
        )
        .unwrap()
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
) -> Addr {
    let infinity_index_code_id = router.store_code(contract_infinity_index());
    let msg = infinity_index::msg::InstantiateMsg {
        infinity_global: infinity_global.to_string(),
    };
    router
        .instantiate_contract(
            infinity_index_code_id,
            creator.clone(),
            &msg,
            &[],
            "Infinity Index",
            None,
        )
        .unwrap()
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
    infinity_global: &Addr,
) -> Addr {
    let infinity_router_code_id = router.store_code(contract_infinity_router());
    let msg = infinity_router::msg::InstantiateMsg {
        infinity_global: infinity_global.to_string(),
    };
    router
        .instantiate_contract(
            infinity_router_code_id,
            creator.clone(),
            &msg,
            &[],
            "Infinity Router",
            None,
        )
        .unwrap()
}

pub fn contract_infinity_pair() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        infinity_pair::execute::execute,
        infinity_pair::instantiate::instantiate,
        infinity_pair::query::query,
    );
    Box::new(contract)
}
