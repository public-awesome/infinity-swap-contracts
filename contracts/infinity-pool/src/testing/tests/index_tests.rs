use std::fmt::Error;
use std::vec;

use crate::error::ContractError;
use crate::execute::execute;
use crate::helpers::{get_next_pool_counter, save_pool};
use crate::instantiate::instantiate;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{pools, BondingCurve, Pool, PoolType, POOL_COUNTER};
use crate::testing::setup::setup_accounts::setup_accounts;
use crate::testing::setup::setup_infinity_pool::setup_infinity_pool;
use crate::testing::setup::setup_marketplace::setup_marketplace;
use crate::testing::setup::templates::standard_minter_template;
use crate::testing::tests::test_helpers::assert_error;
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
fn try_save_pool() {
    let mut deps = mock_dependencies();
    POOL_COUNTER.save(&mut deps.storage, &1).unwrap();

    let pool_counter = get_next_pool_counter(&mut deps.storage).unwrap();
    assert!(pool_counter == 1);

    let pool = Pool::new(
        pool_counter,
        Addr::unchecked(COLLECTION),
        Addr::unchecked(CREATOR),
        Some(Addr::unchecked(ASSET_ACCOUNT)),
        PoolType::Token,
        BondingCurve::Linear,
        Uint128::from(1u128),
        Uint128::from(1u128),
        None,
    );
    assert!(save_pool(&mut deps.storage, &pool).is_ok());
}

#[test]
fn validate_pool_spot_prices() {
    let mut deps = mock_dependencies();
    POOL_COUNTER.save(&mut deps.storage, &1).unwrap();

    // Token Linear pools set the correct spot price
    let pool_counter = get_next_pool_counter(&mut deps.storage).unwrap();
    assert!(pool_counter == 1);
    let test_spot_price = Uint128::from(101u128);
    let pool = Pool::new(
        pool_counter,
        Addr::unchecked(COLLECTION),
        Addr::unchecked(CREATOR),
        Some(Addr::unchecked(ASSET_ACCOUNT)),
        PoolType::Token,
        BondingCurve::Linear,
        test_spot_price,
        Uint128::from(1u128),
        None,
    );
    assert!(save_pool(&mut deps.storage, &pool).is_ok());

    let pool = pools().load(&deps.storage, pool_counter).unwrap();
    assert_eq!(test_spot_price, pool.spot_price);
    assert_eq!(test_spot_price, pool.get_buy_quote().unwrap());

    // Token Exponential pools set the correct spot price
    let pool_counter = get_next_pool_counter(&mut deps.storage).unwrap();
    assert!(pool_counter == 2);
    let test_spot_price = Uint128::from(102u128);
    let pool = Pool::new(
        pool_counter,
        Addr::unchecked(COLLECTION),
        Addr::unchecked(CREATOR),
        Some(Addr::unchecked(ASSET_ACCOUNT)),
        PoolType::Token,
        BondingCurve::Exponential,
        test_spot_price,
        Uint128::from(1u128),
        None,
    );
    assert!(save_pool(&mut deps.storage, &pool).is_ok());

    let pool = pools().load(&deps.storage, pool_counter).unwrap();
    assert_eq!(test_spot_price, pool.spot_price);
    assert_eq!(test_spot_price, pool.get_buy_quote().unwrap());

    // Nft Linear pools set the correct spot price
    let pool_counter = get_next_pool_counter(&mut deps.storage).unwrap();
    assert!(pool_counter == 3);
    let test_spot_price = Uint128::from(103u128);
    let pool = Pool::new(
        pool_counter,
        Addr::unchecked(COLLECTION),
        Addr::unchecked(CREATOR),
        Some(Addr::unchecked(ASSET_ACCOUNT)),
        PoolType::Nft,
        BondingCurve::Linear,
        test_spot_price,
        Uint128::from(1u128),
        None,
    );
    assert!(save_pool(&mut deps.storage, &pool).is_ok());

    let pool = pools().load(&deps.storage, pool_counter).unwrap();
    assert_eq!(test_spot_price, pool.spot_price);
    assert_eq!(test_spot_price, pool.get_sell_quote().unwrap());

    // Nft Exponential pools set the correct spot price
    let pool_counter = get_next_pool_counter(&mut deps.storage).unwrap();
    assert!(pool_counter == 4);
    let test_spot_price = Uint128::from(104u128);
    let pool = Pool::new(
        pool_counter,
        Addr::unchecked(COLLECTION),
        Addr::unchecked(CREATOR),
        Some(Addr::unchecked(ASSET_ACCOUNT)),
        PoolType::Nft,
        BondingCurve::Exponential,
        test_spot_price,
        Uint128::from(1u128),
        None,
    );
    assert!(save_pool(&mut deps.storage, &pool).is_ok());

    let pool = pools().load(&deps.storage, pool_counter).unwrap();
    assert_eq!(test_spot_price, pool.spot_price);
    assert_eq!(test_spot_price, pool.get_sell_quote().unwrap());

    // Trade Linear pools set the correct spot price
    let pool_counter = get_next_pool_counter(&mut deps.storage).unwrap();
    assert!(pool_counter == 5);
    let test_spot_price = Uint128::from(105u128);
    let test_delta = Uint128::from(3u128);
    let pool = Pool::new(
        pool_counter,
        Addr::unchecked(COLLECTION),
        Addr::unchecked(CREATOR),
        Some(Addr::unchecked(ASSET_ACCOUNT)),
        PoolType::Trade,
        BondingCurve::Linear,
        test_spot_price,
        test_delta,
        None,
    );
    assert!(save_pool(&mut deps.storage, &pool).is_ok());

    let pool = pools().load(&deps.storage, pool_counter).unwrap();
    assert_eq!(test_spot_price + test_delta, pool.get_buy_quote().unwrap());
    assert_eq!(test_spot_price, pool.get_sell_quote().unwrap());

    // Trade Exponential pools set the correct spot price
    let pool_counter = get_next_pool_counter(&mut deps.storage).unwrap();
    assert!(pool_counter == 6);
    let test_spot_price = Uint128::from(1550000u128);
    let test_delta = Uint128::from(1023u128);
    let pool = Pool::new(
        pool_counter,
        Addr::unchecked(COLLECTION),
        Addr::unchecked(CREATOR),
        Some(Addr::unchecked(ASSET_ACCOUNT)),
        PoolType::Trade,
        BondingCurve::Exponential,
        test_spot_price,
        test_delta,
        None,
    );
    assert!(save_pool(&mut deps.storage, &pool).is_ok());

    let pool = pools().load(&deps.storage, pool_counter).unwrap();
    assert_eq!(Uint128::from(1708565u128), pool.get_buy_quote().unwrap());
    assert_eq!(test_spot_price, pool.get_sell_quote().unwrap());

    // Trade Constant Product pools set the correct spot price
    let pool_counter = get_next_pool_counter(&mut deps.storage).unwrap();
    assert!(pool_counter == 7);
    let test_spot_price = Uint128::from(105u128);
    let test_delta = Uint128::from(3u128);
    let pool = Pool::new(
        pool_counter,
        Addr::unchecked(COLLECTION),
        Addr::unchecked(CREATOR),
        Some(Addr::unchecked(ASSET_ACCOUNT)),
        PoolType::Trade,
        BondingCurve::ConstantProduct,
        test_spot_price,
        test_delta,
        None,
    );
    assert!(save_pool(&mut deps.storage, &pool).is_ok());

    let pool = pools().load(&deps.storage, pool_counter).unwrap();
    assert_eq!(test_spot_price, pool.spot_price);
}
