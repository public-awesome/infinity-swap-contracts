use crate::helpers::pool_functions::create_pool;
use crate::helpers::utils::{assert_error, assert_event};
use crate::setup::setup_infinity_contracts::{setup_infinity_test, InfinityTestSetup};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_multi_test::Executor;
use infinity_pool::msg::{
    ExecuteMsg as InfinityPoolExecuteMsg, PoolConfigResponse, PoolInfo,
    QueryMsg as InfinityPoolQueryMsg,
};
use infinity_pool::state::{BondingCurve, PoolType};
use infinity_pool::ContractError;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_set_is_active_token_pool() {
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        marketplace,
        infinity_index,
        infinity_pool_code_id,
        ..
    } = setup_infinity_test(100).unwrap();

    let collection_resp = &collection_response_vec[0];
    let collection = collection_resp.collection.clone().unwrap();

    let deposit_amount = Uint128::from(10_000_000u128);

    let infinity_pool = create_pool(
        &mut router,
        infinity_pool_code_id,
        &accts.owner,
        marketplace.to_string(),
        infinity_index.to_string(),
        PoolInfo {
            collection: collection.to_string(),
            owner: accts.owner.to_string(),
            asset_recipient: None,
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(10_000u128),
            delta: Uint128::from(10u128),
            finders_fee_bps: 0u64,
            swap_fee_bps: 0u64,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        deposit_amount,
    );

    // Only owner can set activate a pool
    let response = router.execute_contract(
        accts.bidder.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::SetIsActive {
            is_active: true,
        },
        &[],
    );
    assert_error(
        response,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Owner can update pool config
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::SetIsActive {
            is_active: true,
        },
        &[],
    );
    assert_event(response, "wasm-set-is-active");

    let pool_config = router
        .wrap()
        .query_wasm_smart::<PoolConfigResponse>(
            infinity_pool.clone(),
            &&InfinityPoolQueryMsg::PoolConfig {},
        )
        .unwrap()
        .config;

    assert_eq!(pool_config.is_active, true);

    // Owner can deactive the pool
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::SetIsActive {
            is_active: false,
        },
        &[],
    );
    assert_event(response, "wasm-set-is-active");

    let pool_config = router
        .wrap()
        .query_wasm_smart::<PoolConfigResponse>(
            infinity_pool.clone(),
            &&InfinityPoolQueryMsg::PoolConfig {},
        )
        .unwrap()
        .config;

    assert_eq!(pool_config.is_active, false);
}

#[test]
fn try_set_is_active_nft_pool() {
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        marketplace,
        infinity_index,
        infinity_pool_code_id,
        ..
    } = setup_infinity_test(100).unwrap();

    let collection_resp = &collection_response_vec[0];
    let collection = collection_resp.collection.clone().unwrap();

    let deposit_amount = Uint128::from(10_000_000u128);

    let infinity_pool = create_pool(
        &mut router,
        infinity_pool_code_id,
        &accts.owner,
        marketplace.to_string(),
        infinity_index.to_string(),
        PoolInfo {
            collection: collection.to_string(),
            owner: accts.owner.to_string(),
            asset_recipient: None,
            pool_type: PoolType::Nft,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(10_000u128),
            delta: Uint128::from(10u128),
            finders_fee_bps: 0u64,
            swap_fee_bps: 0u64,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        deposit_amount,
    );

    // Only owner can set activate a pool
    let response = router.execute_contract(
        accts.bidder.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::SetIsActive {
            is_active: true,
        },
        &[],
    );
    assert_error(
        response,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Owner can update pool config
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::SetIsActive {
            is_active: true,
        },
        &[],
    );
    assert_event(response, "wasm-set-is-active");

    let pool_config = router
        .wrap()
        .query_wasm_smart::<PoolConfigResponse>(
            infinity_pool.clone(),
            &&InfinityPoolQueryMsg::PoolConfig {},
        )
        .unwrap()
        .config;

    assert_eq!(pool_config.is_active, true);

    // Owner can deactive the pool
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::SetIsActive {
            is_active: false,
        },
        &[],
    );
    assert_event(response, "wasm-set-is-active");

    let pool_config = router
        .wrap()
        .query_wasm_smart::<PoolConfigResponse>(
            infinity_pool.clone(),
            &&InfinityPoolQueryMsg::PoolConfig {},
        )
        .unwrap()
        .config;

    assert_eq!(pool_config.is_active, false);
}

#[test]
fn try_set_is_active_trade_pool() {
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        marketplace,
        infinity_index,
        infinity_pool_code_id,
        ..
    } = setup_infinity_test(100).unwrap();

    let collection_resp = &collection_response_vec[0];
    let collection = collection_resp.collection.clone().unwrap();

    let deposit_amount = Uint128::from(10_000_000u128);

    let infinity_pool = create_pool(
        &mut router,
        infinity_pool_code_id,
        &accts.owner,
        marketplace.to_string(),
        infinity_index.to_string(),
        PoolInfo {
            collection: collection.to_string(),
            owner: accts.owner.to_string(),
            asset_recipient: None,
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(10_000u128),
            delta: Uint128::from(10u128),
            finders_fee_bps: 0u64,
            swap_fee_bps: 0u64,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        deposit_amount,
    );

    // Only owner can set activate a pool
    let response = router.execute_contract(
        accts.bidder.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::SetIsActive {
            is_active: true,
        },
        &[],
    );
    assert_error(
        response,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Owner can update pool config
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::SetIsActive {
            is_active: true,
        },
        &[],
    );
    assert_event(response, "wasm-set-is-active");

    let pool_config = router
        .wrap()
        .query_wasm_smart::<PoolConfigResponse>(
            infinity_pool.clone(),
            &&InfinityPoolQueryMsg::PoolConfig {},
        )
        .unwrap()
        .config;

    assert_eq!(pool_config.is_active, true);

    // Owner can deactive the pool
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::SetIsActive {
            is_active: false,
        },
        &[],
    );
    assert_event(response, "wasm-set-is-active");

    let pool_config = router
        .wrap()
        .query_wasm_smart::<PoolConfigResponse>(
            infinity_pool.clone(),
            &&InfinityPoolQueryMsg::PoolConfig {},
        )
        .unwrap()
        .config;

    assert_eq!(pool_config.is_active, false);
}
