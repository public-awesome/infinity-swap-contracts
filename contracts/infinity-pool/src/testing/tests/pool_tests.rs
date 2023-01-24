use std::fmt::Error;
use std::vec;

use crate::error::ContractError;
use crate::execute::execute;
use crate::instantiate::instantiate;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{BondingCurve, PoolType};
use crate::testing::helpers::pool_functions::create_pool;
use crate::testing::helpers::utils::assert_error;
use crate::testing::setup::setup_accounts::setup_accounts;
use crate::testing::setup::setup_infinity_pool::setup_infinity_pool;
use crate::testing::setup::setup_marketplace::setup_marketplace;
use crate::testing::setup::templates::standard_minter_template;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, Addr, Attribute, Uint128};
use cw_multi_test::Executor;
use sg_std::NATIVE_DENOM;
use test_suite::common_setup::contract_boxes::custom_mock_app;

const CREATOR: &str = "creator";
const ASSET_ACCOUNT: &str = "asset";
const MARKETPLACE: &str = "marketplace";
const COLLECTION: &str = "collection";
const TOKEN_ID: u32 = 123;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();

    let marketplace_addr = Addr::unchecked(MARKETPLACE);

    let msg = InstantiateMsg {
        denom: NATIVE_DENOM.to_string(),
        marketplace_addr: marketplace_addr.to_string(),
    };
    let info = mock_info("creator", &coins(1000, NATIVE_DENOM));

    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    let expected = vec![
        Attribute {
            key: "denom".to_string(),
            value: NATIVE_DENOM.to_string(),
        },
        Attribute {
            key: "marketplace_addr".to_string(),
            value: marketplace_addr.to_string(),
        },
    ];
    assert_eq!(res.attributes[3..5], expected);
}

#[test]
fn create_token_pool() {
    let vt = standard_minter_template(1);
    let (mut router, creator, bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool =
        setup_infinity_pool(&mut router, creator.clone(), marketplace.clone()).unwrap();

    // Cannot create a ConstantProduct Token Pool because the pool does not hold both assets
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        pool_type: PoolType::Token,
        bonding_curve: BondingCurve::ConstantProduct,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        fee_bps: None,
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::InvalidPool(String::from(
            "constant product bonding curve cannot be used with token pools",
        )),
    );

    // Cannot create a Token Pool with a fee
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        pool_type: PoolType::Token,
        bonding_curve: BondingCurve::Linear,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        fee_bps: Some(0u16),
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::InvalidPool(String::from("fee_bps must be 0 for token pool")),
    );

    // Can create a Linear Token Pool
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        pool_type: PoolType::Token,
        bonding_curve: BondingCurve::Linear,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        fee_bps: None,
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert!(res.is_ok());

    // Can create an Exponential Token Pool
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        pool_type: PoolType::Token,
        bonding_curve: BondingCurve::Exponential,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        fee_bps: None,
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert!(res.is_ok());
}

#[test]
fn deposit_assets_token_pool() {
    let vt = standard_minter_template(1);
    let (mut router, creator, bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool =
        setup_infinity_pool(&mut router, creator.clone(), marketplace.clone()).unwrap();

    let pool_id = create_pool(
        &mut router,
        infinity_pool.clone(),
        creator.clone(),
        collection,
        Some(asset_account),
        PoolType::Token,
        BondingCurve::Linear,
        Uint128::from(2400u64),
        Uint128::from(100u64),
        None,
    )
    .unwrap();

    // Only owner of pool can deposit tokens
    let deposit_amount = 1000u128;
    let msg = ExecuteMsg::DepositTokens { pool_id };
    let res = router.execute_contract(
        bidder.clone(),
        infinity_pool.clone(),
        &msg,
        &coins(deposit_amount, NATIVE_DENOM),
    );
    assert_error(
        res,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Owner can deposit tokens
    let deposit_amount = 1000u128;
    let msg = ExecuteMsg::DepositTokens { pool_id };
    let res = router.execute_contract(
        creator.clone(),
        infinity_pool.clone(),
        &msg,
        &coins(deposit_amount, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Cannot deposit NFTs into token pool
}

#[test]
fn create_nft_pool() {
    let mut app = custom_mock_app();
    let vt = standard_minter_template(1);
    let (mut router, creator, bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool =
        setup_infinity_pool(&mut router, creator.clone(), marketplace.clone()).unwrap();

    // Cannot create a ConstantProduct Nft Pool because the pool does not hold both assets
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        pool_type: PoolType::Nft,
        bonding_curve: BondingCurve::ConstantProduct,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        fee_bps: None,
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::InvalidPool(String::from(
            "constant product bonding curve cannot be used with nft pools",
        )),
    );

    // Cannot create a Nft Pool with a fee
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        pool_type: PoolType::Nft,
        bonding_curve: BondingCurve::Linear,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        fee_bps: Some(0u16),
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::InvalidPool(String::from("fee_bps must be 0 for nft pool")),
    );

    // Can create a Linear Nft Pool
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        pool_type: PoolType::Nft,
        bonding_curve: BondingCurve::Linear,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        fee_bps: None,
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert!(res.is_ok());

    // Can create an Exponential Nft Pool
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        pool_type: PoolType::Nft,
        bonding_curve: BondingCurve::Exponential,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        fee_bps: None,
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert!(res.is_ok());
}

#[test]
fn create_trade_pool() {
    let mut app = custom_mock_app();
    let vt = standard_minter_template(1);
    let (mut router, creator, bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool =
        setup_infinity_pool(&mut router, creator.clone(), marketplace.clone()).unwrap();

    // Cannot create a Trade Pool with a fee > 9000;
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        pool_type: PoolType::Trade,
        bonding_curve: BondingCurve::Linear,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        fee_bps: Some(9001u16),
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::InvalidPool(String::from("fee_bps is greater than 9000")),
    );

    // Can create a Linear Trade Pool w/ no fee
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        pool_type: PoolType::Trade,
        bonding_curve: BondingCurve::Linear,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        fee_bps: None,
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert!(res.is_ok());

    // Can create an Exponential Trade Pool
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        pool_type: PoolType::Trade,
        bonding_curve: BondingCurve::Exponential,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        fee_bps: Some(2000u16),
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert!(res.is_ok());

    // Can create an Constant Product Trade Pool
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        pool_type: PoolType::Trade,
        bonding_curve: BondingCurve::ConstantProduct,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        fee_bps: Some(2000u16),
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert!(res.is_ok());
}
