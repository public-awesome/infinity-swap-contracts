use crate::setup::setup_infinity_contracts::contract_infinity_global;

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_multi_test::Executor;
use infinity_global::msg::{GlobalConfigResponse, InstantiateMsg, QueryMsg, SudoMsg};
use test_suite::common_setup::contract_boxes::custom_mock_app;

#[test]
fn try_infinity_global_messages() {
    let creator = Addr::unchecked("creator");

    let infinity_index = Addr::unchecked("infinity_index");
    let infinity_factory = Addr::unchecked("infinity_factory");
    let min_price = Uint128::from(10u128);
    let pool_creation_fee = Uint128::from(200u128);
    let trading_fee_bps = 500u64;

    let mut router = custom_mock_app();
    let infinity_global_code_id = router.store_code(contract_infinity_global());
    let msg = InstantiateMsg {
        infinity_index: infinity_index.to_string(),
        infinity_factory: infinity_factory.to_string(),
        min_price: min_price.clone(),
        pool_creation_fee: pool_creation_fee.clone(),
        trading_fee_bps: trading_fee_bps.clone(),
    };
    let response = router.instantiate_contract(
        infinity_global_code_id,
        creator.clone(),
        &msg,
        &[],
        "InfinityGlobal",
        None,
    );
    assert!(response.is_ok());
    let infinity_global = response.unwrap();

    let global_config = router
        .wrap()
        .query_wasm_smart::<GlobalConfigResponse>(
            infinity_global.clone(),
            &QueryMsg::GlobalConfig {},
        )
        .unwrap()
        .config;

    assert_eq!(global_config.infinity_index, infinity_index);
    assert_eq!(global_config.infinity_factory, infinity_factory);
    assert_eq!(global_config.min_price, min_price);
    assert_eq!(global_config.pool_creation_fee, pool_creation_fee);
    assert_eq!(global_config.trading_fee_percent, Decimal::percent(trading_fee_bps));

    let infinity_index = Addr::unchecked("infinity_index2");
    let infinity_factory = Addr::unchecked("infinity_factory2");
    let min_price = Uint128::from(20u128);
    let pool_creation_fee = Uint128::from(300u128);
    let trading_fee_bps = 600u64;
    let response = router.wasm_sudo(
        infinity_global.clone(),
        &SudoMsg::UpdateConfig {
            infinity_index: Some(infinity_index.to_string()),
            infinity_factory: Some(infinity_factory.to_string()),
            min_price: Some(min_price),
            pool_creation_fee: Some(pool_creation_fee),
            trading_fee_bps: Some(trading_fee_bps),
        },
    );
    assert!(response.is_ok());

    let global_config = router
        .wrap()
        .query_wasm_smart::<GlobalConfigResponse>(infinity_global, &QueryMsg::GlobalConfig {})
        .unwrap()
        .config;

    assert_eq!(global_config.infinity_index, infinity_index);
    assert_eq!(global_config.infinity_factory, infinity_factory);
    assert_eq!(global_config.min_price, min_price);
    assert_eq!(global_config.pool_creation_fee, pool_creation_fee);
    assert_eq!(global_config.trading_fee_percent, Decimal::percent(trading_fee_bps));
}
