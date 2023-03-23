use std::vec;

use crate::helpers::nft_functions::{approve, mint};
use crate::helpers::pool_functions::create_pool;
use crate::helpers::utils::assert_error;
use crate::setup::setup_accounts::setup_addtl_account;
use crate::setup::setup_infinity_swap::setup_infinity_swap;
use crate::setup::setup_marketplace::{setup_marketplace, LISTING_FEE};
use crate::setup::templates::standard_minter_template;
use cosmwasm_std::{coins, Addr, Uint128};
use cw_multi_test::Executor;
use infinity_swap::msg::ExecuteMsg;
use infinity_swap::state::BondingCurve;
use infinity_swap::ContractError;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;

const ASSET_ACCOUNT: &str = "asset";

#[test]
fn create_nft_pool() {
    let vt = standard_minter_template(5000);
    let (mut router, creator, _bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_swap = setup_infinity_swap(&mut router, creator.clone(), marketplace).unwrap();

    // Cannot create a ConstantProduct Nft Pool because the pool does not hold both assets
    let msg = ExecuteMsg::CreateNftPool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        bonding_curve: BondingCurve::ConstantProduct,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        finders_fee_bps: 0,
    };
    let res = router.execute_contract(creator.clone(), infinity_swap.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::InvalidPool(String::from(
            "constant product bonding curve cannot be used with nft pools",
        )),
    );

    // Can create a Linear Nft Pool
    let msg = ExecuteMsg::CreateNftPool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        bonding_curve: BondingCurve::Linear,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        finders_fee_bps: 0,
    };
    let res = router.execute_contract(
        creator.clone(),
        infinity_swap.clone(),
        &msg,
        &coins(LISTING_FEE, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    // Can create an Exponential Nft Pool
    let msg = ExecuteMsg::CreateNftPool {
        collection: collection.to_string(),
        asset_recipient: Some(asset_account.to_string()),
        bonding_curve: BondingCurve::Exponential,
        spot_price: Uint128::from(2400u64),
        delta: Uint128::from(120u64),
        finders_fee_bps: 0,
    };
    let res = router.execute_contract(
        creator,
        infinity_swap,
        &msg,
        &coins(LISTING_FEE, NATIVE_DENOM),
    );
    assert!(res.is_ok());
}

#[test]
fn deposit_assets_nft_pool() {
    let vt = standard_minter_template(5000);
    let (mut router, minter, creator, user1) = (
        vt.router,
        vt.collection_response_vec[0].minter.as_ref().unwrap(),
        vt.accts.creator,
        vt.accts.bidder,
    );
    let _user2 = setup_addtl_account(&mut router, "bidder2", 5_000_002_000);

    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);
    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_swap = setup_infinity_swap(&mut router, creator.clone(), marketplace).unwrap();

    let pool = create_pool(
        &mut router,
        infinity_swap.clone(),
        creator.clone(),
        ExecuteMsg::CreateNftPool {
            collection: collection.to_string(),
            asset_recipient: Some(asset_account.to_string()),
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(2400u64),
            delta: Uint128::from(100u64),
            finders_fee_bps: 0,
        },
    )
    .unwrap();

    // Only owner can deposit NFTs into nft pool
    let token_id_1 = mint(&mut router, &user1, minter);
    approve(&mut router, &user1, &collection, &infinity_swap, token_id_1);
    let token_id_2 = mint(&mut router, &user1, minter);
    approve(&mut router, &user1, &collection, &infinity_swap, token_id_2);
    let msg = ExecuteMsg::DepositNfts {
        pool_id: pool.id,
        collection: collection.to_string(),
        nft_token_ids: vec![token_id_1.to_string(), token_id_2.to_string()],
    };
    let res = router.execute_contract(user1.clone(), infinity_swap.clone(), &msg, &[]);
    assert_error(
        res,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Owner can deposit NFTs into nft pool
    let token_id_1 = mint(&mut router, &creator, minter);
    approve(
        &mut router,
        &creator,
        &collection,
        &infinity_swap,
        token_id_1,
    );
    let token_id_2 = mint(&mut router, &creator, minter);
    approve(
        &mut router,
        &creator,
        &collection,
        &infinity_swap,
        token_id_2,
    );
    let msg = ExecuteMsg::DepositNfts {
        pool_id: pool.id,
        collection: collection.to_string(),
        nft_token_ids: vec![token_id_1.to_string(), token_id_2.to_string()],
    };
    let res = router.execute_contract(creator.clone(), infinity_swap.clone(), &msg, &[]);
    assert!(res.is_ok());

    // Owner cannot deposit tokens into nft pool
    let deposit_amount = 1000u128;
    let msg = ExecuteMsg::DepositTokens { pool_id: pool.id };
    let res = router.execute_contract(
        creator,
        infinity_swap,
        &msg,
        &coins(deposit_amount, NATIVE_DENOM),
    );
    assert_error(
        res,
        ContractError::InvalidPool("cannot deposit tokens into nft pool".to_string()),
    );
}
