use std::vec;

use crate::error::ContractError;
use crate::msg::ExecuteMsg;
use crate::state::{BondingCurve, PoolType};
use crate::testing::helpers::nft_functions::{approve, mint};
use crate::testing::helpers::pool_functions::{create_pool, deposit_tokens};
use crate::testing::helpers::utils::assert_error;
use crate::testing::setup::setup_accounts::{setup_second_bidder_account, INITIAL_BALANCE};
use crate::testing::setup::setup_infinity_pool::setup_infinity_pool;
use crate::testing::setup::setup_marketplace::setup_marketplace;
use crate::testing::setup::templates::standard_minter_template;
use cosmwasm_std::{coins, Addr, Attribute, Uint128};
use cw_multi_test::Executor;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;

const ASSET_ACCOUNT: &str = "asset";

#[test]
fn create_token_pool() {
    let vt = standard_minter_template(5000);
    let (mut router, creator, _bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

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
    let res = router.execute_contract(creator, infinity_pool, &msg, &[]);
    assert!(res.is_ok());
}

#[test]
fn deposit_assets_token_pool() {
    let vt = standard_minter_template(5000);
    let (mut router, minter, creator, user1) = (
        vt.router,
        vt.collection_response_vec[0].minter.as_ref().unwrap(),
        vt.accts.creator,
        vt.accts.bidder,
    );
    let _user2 = setup_second_bidder_account(&mut router);

    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);
    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    let pool_id = create_pool(
        &mut router,
        infinity_pool.clone(),
        creator.clone(),
        collection.clone(),
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
        user1,
        infinity_pool.clone(),
        &msg,
        &coins(deposit_amount, NATIVE_DENOM),
    );
    assert_error(
        res,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Owner can deposit tokens
    let deposit_amount_1 = 1250u128;
    let msg = ExecuteMsg::DepositTokens { pool_id };
    let res = router.execute_contract(
        creator.clone(),
        infinity_pool.clone(),
        &msg,
        &coins(deposit_amount_1, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Tokens are summed by the pool
    let deposit_amount_2 = 3200u128;
    let total_tokens = deposit_tokens(
        &mut router,
        infinity_pool.clone(),
        creator.clone(),
        pool_id,
        Uint128::from(deposit_amount_2),
    )
    .unwrap();
    assert_eq!(deposit_amount_1 + deposit_amount_2, total_tokens);

    // Cannot deposit NFTs into token pool
    let token_id_1 = mint(&mut router, &creator, minter);
    approve(
        &mut router,
        &creator,
        &collection,
        &infinity_pool,
        token_id_1,
    );
    let token_id_2 = mint(&mut router, &creator, minter);
    approve(
        &mut router,
        &creator,
        &collection,
        &infinity_pool,
        token_id_2,
    );
    let msg = ExecuteMsg::DepositNfts {
        pool_id,
        collection: collection.to_string(),
        nft_token_ids: vec![token_id_1.to_string(), token_id_2.to_string()],
    };
    let res = router.execute_contract(creator, infinity_pool, &msg, &[]);
    assert_error(
        res,
        ContractError::InvalidPool("cannot deposit nfts into token pool".to_string()),
    );
}

#[test]
fn withdraw_assets_token_pool() {
    let vt = standard_minter_template(5000);
    let (mut router, _minter, creator, user1) = (
        vt.router,
        vt.collection_response_vec[0].minter.as_ref().unwrap(),
        vt.accts.creator,
        vt.accts.bidder,
    );

    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);
    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    let pool_id = create_pool(
        &mut router,
        infinity_pool.clone(),
        creator.clone(),
        collection,
        Some(asset_account.clone()),
        PoolType::Token,
        BondingCurve::Linear,
        Uint128::from(2400u64),
        Uint128::from(100u64),
        None,
    )
    .unwrap();

    let deposit_amount = Uint128::from(1000u64);
    deposit_tokens(
        &mut router,
        infinity_pool.clone(),
        creator.clone(),
        pool_id,
        deposit_amount,
    )
    .unwrap();

    // Only owner of pool can withdraw tokens
    let msg = ExecuteMsg::WithdrawTokens {
        pool_id,
        amount: Uint128::from(300u64),
        asset_recipient: None,
    };
    let res = router.execute_contract(user1, infinity_pool.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Owner of pool can withdraw tokens, tokens are directed to asset_recipient
    let withdraw_amount = Uint128::from(300u64);
    let msg = ExecuteMsg::WithdrawTokens {
        pool_id,
        amount: withdraw_amount,
        asset_recipient: Some(asset_account.to_string()),
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    let total_withdrawn = res.as_ref().unwrap().events[1].attributes[2]
        .value
        .parse::<u128>()
        .unwrap();

    let total_tokens = res.as_ref().unwrap().events[1].attributes[3]
        .value
        .parse::<u128>()
        .unwrap();
    assert!(res.is_ok());
    assert_eq!(Uint128::from(total_withdrawn), withdraw_amount);
    assert_eq!(
        Uint128::from(total_tokens),
        deposit_amount - withdraw_amount
    );
    let asset_account_balance = router.wrap().query_all_balances(asset_account).unwrap();
    assert_eq!(withdraw_amount, asset_account_balance[0].amount);

    // Owner of pool can withdraw remaining tokens, tokens are directed toward owner
    let msg = ExecuteMsg::WithdrawAllTokens {
        pool_id,
        asset_recipient: None,
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    let total_withdrawn = res.as_ref().unwrap().events[1].attributes[2]
        .value
        .parse::<u128>()
        .unwrap();
    let total_tokens = res.as_ref().unwrap().events[1].attributes[3]
        .value
        .parse::<u128>()
        .unwrap();
    assert!(res.is_ok());
    assert_eq!(
        Uint128::from(total_withdrawn),
        deposit_amount - withdraw_amount
    );
    assert_eq!(Uint128::from(total_tokens), Uint128::from(0u128));
    let creator_balance = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(
        creator_balance[0].amount,
        Uint128::from(INITIAL_BALANCE) - withdraw_amount
    );

    // Owner of pool cannot withdraw NFTs from a token pool
    let msg = ExecuteMsg::WithdrawNfts {
        pool_id,
        nft_token_ids: vec!["1".to_string()],
        asset_recipient: None,
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::InvalidPool("cannot withdraw nfts from token pool".to_string()),
    );

    // Owner of pool cannot withdraw NFTs from a token pool
    let msg = ExecuteMsg::WithdrawAllNfts {
        pool_id,
        asset_recipient: None,
    };
    let res = router.execute_contract(creator, infinity_pool, &msg, &[]);
    assert_error(
        res,
        ContractError::InvalidPool("cannot withdraw nfts from token pool".to_string()),
    );
}

#[test]
fn update_token_pool() {
    let vt = standard_minter_template(5000);
    let (mut router, _minter, creator, user1) = (
        vt.router,
        vt.collection_response_vec[0].minter.as_ref().unwrap(),
        vt.accts.creator,
        vt.accts.bidder,
    );

    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);
    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    let pool_id = create_pool(
        &mut router,
        infinity_pool.clone(),
        creator.clone(),
        collection,
        None,
        PoolType::Token,
        BondingCurve::Linear,
        Uint128::from(2400u64),
        Uint128::from(100u64),
        None,
    )
    .unwrap();

    // Only owner of pool can update pool
    let msg = ExecuteMsg::UpdatePoolConfig {
        pool_id,
        asset_recipient: Some(asset_account.to_string()),
        delta: Some(Uint128::from(101u64)),
        spot_price: Some(Uint128::from(102u64)),
        fee_bps: None,
    };
    let res = router.execute_contract(user1, infinity_pool.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Fee cannot be set on token pool
    let msg = ExecuteMsg::UpdatePoolConfig {
        pool_id,
        asset_recipient: Some(asset_account.to_string()),
        delta: Some(Uint128::from(101u64)),
        spot_price: Some(Uint128::from(102u64)),
        fee_bps: Some(1000),
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::InvalidPool("fee_bps must be 0 for token pool".to_string()),
    );

    // Properties on pool are updated correctly
    let new_spot_price = Uint128::from(2400u64);
    let new_delta = Uint128::from(100u64);
    let msg = ExecuteMsg::UpdatePoolConfig {
        pool_id,
        asset_recipient: Some(asset_account.to_string()),
        spot_price: Some(new_spot_price),
        delta: Some(new_delta),
        fee_bps: None,
    };
    let res = router.execute_contract(creator, infinity_pool, &msg, &[]);
    assert!(res.is_ok());
    let attrs = &res.as_ref().unwrap().events[1].attributes;
    assert_eq!(
        attrs[1],
        Attribute {
            key: "id".to_string(),
            value: pool_id.to_string()
        }
    );
    assert_eq!(
        attrs[4],
        Attribute {
            key: "asset_recipient".to_string(),
            value: ASSET_ACCOUNT.to_string()
        }
    );
    assert_eq!(
        attrs[7],
        Attribute {
            key: "spot_price".to_string(),
            value: new_spot_price.to_string()
        }
    );
    assert_eq!(
        attrs[8],
        Attribute {
            key: "delta".to_string(),
            value: new_delta.to_string()
        }
    );
}

#[test]
fn remove_token_pool() {
    let vt = standard_minter_template(5000);
    let (mut router, _minter, creator, user1) = (
        vt.router,
        vt.collection_response_vec[0].minter.as_ref().unwrap(),
        vt.accts.creator,
        vt.accts.bidder,
    );

    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);
    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    let pool_id = create_pool(
        &mut router,
        infinity_pool.clone(),
        creator.clone(),
        collection,
        None,
        PoolType::Token,
        BondingCurve::Linear,
        Uint128::from(2400u64),
        Uint128::from(100u64),
        None,
    )
    .unwrap();

    let deposit_amount = Uint128::from(1000u64);
    deposit_tokens(
        &mut router,
        infinity_pool.clone(),
        creator.clone(),
        pool_id,
        deposit_amount,
    )
    .unwrap();

    // Only owner of pool can remove pool
    let msg = ExecuteMsg::RemovePool {
        pool_id,
        asset_recipient: Some(asset_account.to_string()),
    };
    let res = router.execute_contract(user1, infinity_pool.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Owner of pool can remove pool, and asset_recipient is transferred remaining tokens
    let msg = ExecuteMsg::RemovePool {
        pool_id,
        asset_recipient: Some(asset_account.to_string()),
    };
    let res = router.execute_contract(creator, infinity_pool, &msg, &[]);
    assert!(res.is_ok());
    let asset_account_balance = router.wrap().query_all_balances(asset_account).unwrap();
    assert_eq!(deposit_amount, asset_account_balance[0].amount);
}

#[test]
fn activate_token_pool() {
    let vt = standard_minter_template(5000);
    let (mut router, _minter, creator, user1) = (
        vt.router,
        vt.collection_response_vec[0].minter.as_ref().unwrap(),
        vt.accts.creator,
        vt.accts.bidder,
    );

    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let _asset_account = Addr::unchecked(ASSET_ACCOUNT);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);
    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    let pool_id = create_pool(
        &mut router,
        infinity_pool.clone(),
        creator.clone(),
        collection,
        None,
        PoolType::Token,
        BondingCurve::Linear,
        Uint128::from(2400u64),
        Uint128::from(100u64),
        None,
    )
    .unwrap();

    let deposit_amount = Uint128::from(1000u64);
    deposit_tokens(
        &mut router,
        infinity_pool.clone(),
        creator.clone(),
        pool_id,
        deposit_amount,
    )
    .unwrap();

    // Only owner of pool can activate pool
    let msg = ExecuteMsg::SetActivePool {
        is_active: true,
        pool_id,
    };
    let res = router.execute_contract(user1, infinity_pool.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Owner of pool can activate pool
    let msg = ExecuteMsg::SetActivePool {
        is_active: true,
        pool_id,
    };
    let res = router.execute_contract(creator, infinity_pool, &msg, &[]);
    let is_active = &res.as_ref().unwrap().events[1].attributes[2].value;
    assert_eq!(is_active, &"true");
}
