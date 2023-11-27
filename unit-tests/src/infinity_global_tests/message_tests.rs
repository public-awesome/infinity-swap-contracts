use crate::setup::setup_infinity_contracts::{contract_infinity_global, UOSMO};

use cosmwasm_std::{coin, Addr, Coin, Decimal};
use cw_multi_test::Executor;
use infinity_global::{
    msg::{InstantiateMsg, QueryMsg, SudoMsg},
    GlobalConfig,
};
use sg_multi_test::mock_deps;
use sg_std::NATIVE_DENOM;
use test_suite::common_setup::contract_boxes::custom_mock_app;

#[test]
fn try_infinity_global_init() {
    let creator = Addr::unchecked("creator");

    let mut router = custom_mock_app();
    let infinity_global_code_id = router.store_code(contract_infinity_global());

    let fair_burn = Addr::unchecked("fair_burn");
    let royalty_registry = Addr::unchecked("royalty_registry");
    let marketplace = Addr::unchecked("marketplace");
    let infinity_index = Addr::unchecked("infinity_index");
    let infinity_factory = Addr::unchecked("infinity_factory");
    let infinity_router = Addr::unchecked("infinity_router");

    let global_config = GlobalConfig {
        fair_burn: fair_burn.to_string(),
        royalty_registry: royalty_registry.to_string(),
        marketplace: marketplace.to_string(),
        infinity_factory: infinity_factory.to_string(),
        infinity_index: infinity_index.to_string(),
        infinity_router: infinity_router.to_string(),
        infinity_pair_code_id: 1u64,
        pair_creation_fee: coin(1_000_000u128, NATIVE_DENOM),
        fair_burn_fee_percent: Decimal::percent(1u64),
        default_royalty_fee_percent: Decimal::percent(10u64),
        max_royalty_fee_percent: Decimal::percent(15u64),
        max_swap_fee_percent: Decimal::percent(10u64),
    };

    let min_prices = vec![coin(1_000_000u128, NATIVE_DENOM)];

    let msg = InstantiateMsg {
        global_config: global_config.clone(),
        min_prices: min_prices.clone(),
    };
    let response = router.instantiate_contract(
        infinity_global_code_id,
        creator,
        &msg,
        &[],
        "Infinity Global",
        None,
    );
    assert!(response.is_ok());
    let infinity_global = response.unwrap();

    let global_config_response = router
        .wrap()
        .query_wasm_smart::<GlobalConfig<Addr>>(infinity_global.clone(), &QueryMsg::GlobalConfig {})
        .unwrap();

    let deps = mock_deps();
    assert_eq!(global_config.str_to_addr(&deps.api).unwrap(), global_config_response);

    let min_price_response = router
        .wrap()
        .query_wasm_smart::<Option<Coin>>(
            infinity_global,
            &QueryMsg::MinPrice {
                denom: NATIVE_DENOM.to_string(),
            },
        )
        .unwrap();
    assert_eq!(min_prices[0], min_price_response.unwrap());
}

#[test]
fn try_infinity_global_update_config() {
    let creator = Addr::unchecked("creator");

    let mut router = custom_mock_app();
    let infinity_global_code_id = router.store_code(contract_infinity_global());

    let fair_burn = Addr::unchecked("fair_burn");
    let royalty_registry = Addr::unchecked("royalty_registry");
    let marketplace = Addr::unchecked("marketplace");
    let infinity_index = Addr::unchecked("infinity_index");
    let infinity_factory = Addr::unchecked("infinity_factory");
    let infinity_router = Addr::unchecked("infinity_router");

    let global_config = GlobalConfig {
        fair_burn: fair_burn.to_string(),
        royalty_registry: royalty_registry.to_string(),
        marketplace: marketplace.to_string(),
        infinity_factory: infinity_factory.to_string(),
        infinity_index: infinity_index.to_string(),
        infinity_router: infinity_router.to_string(),
        infinity_pair_code_id: 1u64,
        pair_creation_fee: coin(1_000_000u128, NATIVE_DENOM),
        fair_burn_fee_percent: Decimal::percent(1u64),
        default_royalty_fee_percent: Decimal::percent(10u64),
        max_royalty_fee_percent: Decimal::percent(15u64),
        max_swap_fee_percent: Decimal::percent(10u64),
    };

    let min_prices = vec![coin(1_000_000u128, NATIVE_DENOM)];

    let msg = InstantiateMsg {
        global_config,
        min_prices,
    };
    let response = router.instantiate_contract(
        infinity_global_code_id,
        creator,
        &msg,
        &[],
        "Infinity Global",
        None,
    );
    assert!(response.is_ok());
    let infinity_global = response.unwrap();

    let update_config_msg = SudoMsg::UpdateConfig {
        fair_burn: Some("fair_burn_new".to_string()),
        royalty_registry: Some("royalty_registry_new".to_string()),
        marketplace: Some("marketplace_new".to_string()),
        infinity_factory: Some("infinity_factory_new".to_string()),
        infinity_index: Some("infinity_index_new".to_string()),
        infinity_router: Some("infinity_router_new".to_string()),
        infinity_pair_code_id: Some(2u64),
        pair_creation_fee: Some(coin(2_000_000u128, NATIVE_DENOM)),
        fair_burn_fee_percent: Some(Decimal::percent(2u64)),
        default_royalty_fee_percent: Some(Decimal::percent(1u64)),
        max_royalty_fee_percent: Some(Decimal::percent(20u64)),
        max_swap_fee_percent: Some(Decimal::percent(20u64)),
    };
    let response = router.wasm_sudo(infinity_global.clone(), &update_config_msg);
    assert!(response.is_ok());

    let global_config_response = router
        .wrap()
        .query_wasm_smart::<GlobalConfig<Addr>>(infinity_global, &QueryMsg::GlobalConfig {})
        .unwrap();

    if let SudoMsg::UpdateConfig {
        fair_burn,
        royalty_registry,
        marketplace,
        infinity_factory,
        infinity_index,
        infinity_router,
        infinity_pair_code_id,
        pair_creation_fee,
        fair_burn_fee_percent,
        default_royalty_fee_percent,
        max_royalty_fee_percent,
        max_swap_fee_percent,
    } = update_config_msg
    {
        assert_eq!(fair_burn.unwrap(), global_config_response.fair_burn);
        assert_eq!(royalty_registry.unwrap(), global_config_response.royalty_registry);
        assert_eq!(marketplace.unwrap(), global_config_response.marketplace);
        assert_eq!(infinity_factory.unwrap(), global_config_response.infinity_factory);
        assert_eq!(infinity_index.unwrap(), global_config_response.infinity_index);
        assert_eq!(infinity_router.unwrap(), global_config_response.infinity_router);
        assert_eq!(infinity_pair_code_id.unwrap(), global_config_response.infinity_pair_code_id);
        assert_eq!(pair_creation_fee.unwrap(), global_config_response.pair_creation_fee);
        assert_eq!(fair_burn_fee_percent.unwrap(), global_config_response.fair_burn_fee_percent);
        assert_eq!(
            default_royalty_fee_percent.unwrap(),
            global_config_response.default_royalty_fee_percent
        );
        assert_eq!(
            max_royalty_fee_percent.unwrap(),
            global_config_response.max_royalty_fee_percent
        );
        assert_eq!(max_swap_fee_percent.unwrap(), global_config_response.max_swap_fee_percent);
    }
}

#[test]
fn try_infinity_global_add_remove_min_prices() {
    let creator = Addr::unchecked("creator");

    let mut router = custom_mock_app();
    let infinity_global_code_id = router.store_code(contract_infinity_global());

    let fair_burn = Addr::unchecked("fair_burn");
    let royalty_registry = Addr::unchecked("royalty_registry");
    let marketplace = Addr::unchecked("marketplace");
    let infinity_index = Addr::unchecked("infinity_index");
    let infinity_factory = Addr::unchecked("infinity_factory");
    let infinity_router = Addr::unchecked("infinity_router");

    let global_config = GlobalConfig {
        fair_burn: fair_burn.to_string(),
        royalty_registry: royalty_registry.to_string(),
        marketplace: marketplace.to_string(),
        infinity_factory: infinity_factory.to_string(),
        infinity_index: infinity_index.to_string(),
        infinity_router: infinity_router.to_string(),
        infinity_pair_code_id: 1u64,
        pair_creation_fee: coin(1_000_000u128, NATIVE_DENOM),
        fair_burn_fee_percent: Decimal::percent(1u64),
        default_royalty_fee_percent: Decimal::percent(10u64),
        max_royalty_fee_percent: Decimal::percent(15u64),
        max_swap_fee_percent: Decimal::percent(10u64),
    };

    let min_prices = vec![coin(1_000_000u128, NATIVE_DENOM)];

    let msg = InstantiateMsg {
        global_config,
        min_prices,
    };
    let response = router.instantiate_contract(
        infinity_global_code_id,
        creator,
        &msg,
        &[],
        "Infinity Global",
        None,
    );
    assert!(response.is_ok());
    let infinity_global = response.unwrap();

    let new_min_prices = vec![coin(3_000_000u128, NATIVE_DENOM), coin(4_000_000u128, UOSMO)];
    let add_min_prices_msg = SudoMsg::AddMinPrices {
        min_prices: new_min_prices.clone(),
    };
    let response = router.wasm_sudo(infinity_global.clone(), &add_min_prices_msg);
    assert!(response.is_ok());

    for min_price in &new_min_prices {
        let min_price_response = router
            .wrap()
            .query_wasm_smart::<Option<Coin>>(
                infinity_global.clone(),
                &QueryMsg::MinPrice {
                    denom: min_price.denom.to_string(),
                },
            )
            .unwrap();
        assert_eq!(min_price, &min_price_response.unwrap());
    }

    let remove_min_prices_msg = SudoMsg::RemoveMinPrices {
        denoms: vec![UOSMO.to_string()],
    };
    let response = router.wasm_sudo(infinity_global.clone(), &remove_min_prices_msg);
    assert!(response.is_ok());

    let min_price_response = router
        .wrap()
        .query_wasm_smart::<Option<Coin>>(
            infinity_global,
            &QueryMsg::MinPrice {
                denom: UOSMO.to_string(),
            },
        )
        .unwrap();
    assert_eq!(None, min_price_response);
}
