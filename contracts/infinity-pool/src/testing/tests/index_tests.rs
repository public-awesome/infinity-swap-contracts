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
        0u16,
    );
    assert!(save_pool(&mut deps.storage, &pool).is_ok());
}

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
        0u16,
    );
    assert!(save_pool(&mut deps.storage, &pool).is_ok());
}

#[test]
fn validate_pool_spot_prices() {
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
        Some(Uint128::from(1u128)),
        Some(Uint128::from(1u128)),
        Some(0u16),
    );
    assert!(save_pool(&mut deps.storage, &pool).is_ok());
}
