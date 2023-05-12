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
fn try_update_pool_config_token_pool() {
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

    let asset_recipient = Addr::unchecked("recipient");
    let spot_price = Uint128::from(20_000u128);
    let delta = Uint128::from(20u128);
    let finders_fee_bps = 10u64;
    let swap_fee_bps = 0u64;
    let reinvest_tokens = false;
    let reinvest_nfts = false;

    // Only owner can update pool config
    let response = router.execute_contract(
        accts.bidder.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::UpdatePoolConfig {
            asset_recipient: Some(asset_recipient.to_string()),
            delta: Some(delta),
            spot_price: Some(spot_price),
            finders_fee_bps: Some(finders_fee_bps),
            swap_fee_bps: Some(swap_fee_bps),
            reinvest_tokens: Some(reinvest_tokens),
            reinvest_nfts: Some(reinvest_nfts),
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
        &InfinityPoolExecuteMsg::UpdatePoolConfig {
            asset_recipient: Some(asset_recipient.to_string()),
            delta: Some(delta),
            spot_price: Some(spot_price),
            finders_fee_bps: Some(finders_fee_bps),
            swap_fee_bps: Some(swap_fee_bps),
            reinvest_tokens: Some(reinvest_tokens),
            reinvest_nfts: Some(reinvest_nfts),
        },
        &[],
    );
    assert_event(response, "wasm-update-pool-config");

    let pool_config = router
        .wrap()
        .query_wasm_smart::<PoolConfigResponse>(
            infinity_pool.clone(),
            &&InfinityPoolQueryMsg::PoolConfig {},
        )
        .unwrap()
        .config;

    assert_eq!(pool_config.asset_recipient, Some(asset_recipient));
    assert_eq!(pool_config.spot_price, spot_price);
    assert_eq!(pool_config.delta, delta);
    assert_eq!(pool_config.finders_fee_percent, Decimal::percent(finders_fee_bps));
    assert_eq!(pool_config.swap_fee_percent, Decimal::percent(swap_fee_bps));
    assert_eq!(pool_config.reinvest_tokens, reinvest_tokens);
    assert_eq!(pool_config.reinvest_nfts, reinvest_nfts);
}

#[test]
fn try_update_pool_config_nft_pool() {
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

    let asset_recipient = Addr::unchecked("recipient");
    let spot_price = Uint128::from(20_000u128);
    let delta = Uint128::from(20u128);
    let finders_fee_bps = 10u64;
    let swap_fee_bps = 0u64;
    let reinvest_tokens = false;
    let reinvest_nfts = false;

    // Only owner can update pool config
    let response = router.execute_contract(
        accts.bidder.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::UpdatePoolConfig {
            asset_recipient: Some(asset_recipient.to_string()),
            delta: Some(delta),
            spot_price: Some(spot_price),
            finders_fee_bps: Some(finders_fee_bps),
            swap_fee_bps: Some(swap_fee_bps),
            reinvest_tokens: Some(reinvest_tokens),
            reinvest_nfts: Some(reinvest_nfts),
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
        &InfinityPoolExecuteMsg::UpdatePoolConfig {
            asset_recipient: Some(asset_recipient.to_string()),
            delta: Some(delta),
            spot_price: Some(spot_price),
            finders_fee_bps: Some(finders_fee_bps),
            swap_fee_bps: Some(swap_fee_bps),
            reinvest_tokens: Some(reinvest_tokens),
            reinvest_nfts: Some(reinvest_nfts),
        },
        &[],
    );
    assert_event(response, "wasm-update-pool-config");

    let pool_config = router
        .wrap()
        .query_wasm_smart::<PoolConfigResponse>(
            infinity_pool.clone(),
            &&InfinityPoolQueryMsg::PoolConfig {},
        )
        .unwrap()
        .config;

    assert_eq!(pool_config.asset_recipient, Some(asset_recipient));
    assert_eq!(pool_config.spot_price, spot_price);
    assert_eq!(pool_config.delta, delta);
    assert_eq!(pool_config.finders_fee_percent, Decimal::percent(finders_fee_bps));
    assert_eq!(pool_config.swap_fee_percent, Decimal::percent(swap_fee_bps));
    assert_eq!(pool_config.reinvest_tokens, reinvest_tokens);
    assert_eq!(pool_config.reinvest_nfts, reinvest_nfts);
}

#[test]
fn try_update_pool_config_trade_pool() {
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

    let asset_recipient = Addr::unchecked("recipient");
    let spot_price = Uint128::from(20_000u128);
    let delta = Uint128::from(20u128);
    let finders_fee_bps = 10u64;
    let swap_fee_bps = 10u64;
    let reinvest_tokens = true;
    let reinvest_nfts = true;

    // Only owner can update pool config
    let response = router.execute_contract(
        accts.bidder.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::UpdatePoolConfig {
            asset_recipient: Some(asset_recipient.to_string()),
            delta: Some(delta),
            spot_price: Some(spot_price),
            finders_fee_bps: Some(finders_fee_bps),
            swap_fee_bps: Some(swap_fee_bps),
            reinvest_tokens: Some(reinvest_tokens),
            reinvest_nfts: Some(reinvest_nfts),
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
        &InfinityPoolExecuteMsg::UpdatePoolConfig {
            asset_recipient: Some(asset_recipient.to_string()),
            delta: Some(delta),
            spot_price: Some(spot_price),
            finders_fee_bps: Some(finders_fee_bps),
            swap_fee_bps: Some(swap_fee_bps),
            reinvest_tokens: Some(reinvest_tokens),
            reinvest_nfts: Some(reinvest_nfts),
        },
        &[],
    );
    assert_event(response, "wasm-update-pool-config");

    let pool_config = router
        .wrap()
        .query_wasm_smart::<PoolConfigResponse>(
            infinity_pool.clone(),
            &&InfinityPoolQueryMsg::PoolConfig {},
        )
        .unwrap()
        .config;

    assert_eq!(pool_config.asset_recipient, Some(asset_recipient));
    assert_eq!(pool_config.spot_price, spot_price);
    assert_eq!(pool_config.delta, delta);
    assert_eq!(pool_config.finders_fee_percent, Decimal::percent(finders_fee_bps));
    assert_eq!(pool_config.swap_fee_percent, Decimal::percent(swap_fee_bps));
    assert_eq!(pool_config.reinvest_tokens, reinvest_tokens);
    assert_eq!(pool_config.reinvest_nfts, reinvest_nfts);
}
