use std::vec;

use crate::msg::QueryMsg::SimDirectSwapNftsForTokens;
use crate::msg::SwapResponse;
use crate::msg::{self, ExecuteMsg};
use crate::msg::{NftSwap, SwapParams};
use crate::state::Pool;
use crate::state::{BondingCurve, PoolType};
use crate::testing::helpers::nft_functions::{approve, mint};
use crate::testing::helpers::pool_functions::create_pool;
use crate::testing::setup::setup_accounts::setup_second_bidder_account;
use crate::testing::setup::setup_infinity_pool::setup_infinity_pool;
use crate::testing::setup::setup_marketplace::setup_marketplace;
use crate::testing::setup::templates::standard_minter_template;
use cosmwasm_std::StdError;
use cosmwasm_std::StdResult;
use cosmwasm_std::Timestamp;
use cosmwasm_std::{coins, Addr, Uint128};
use cw_multi_test::Executor;
use sg_multi_test::StargazeApp;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;

const ASSET_ACCOUNT: &str = "asset";

fn setup_swap_pool() -> (StargazeApp, Addr, Pool, msg::QueryMsg, Addr) {
    let vt = standard_minter_template(5000);
    let (mut router, minter, creator, user1) = (
        vt.router,
        vt.collection_response_vec[0].minter.as_ref().unwrap(),
        vt.accts.creator,
        vt.accts.bidder,
    );
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);
    let _user2 = setup_second_bidder_account(&mut router).unwrap();

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);
    // Can create a Linear Nft Pool
    let pool = create_pool(
        &mut router,
        infinity_pool.clone(),
        creator.clone(),
        ExecuteMsg::CreatePool {
            collection: collection.to_string(),
            asset_recipient: Some(asset_account.to_string()),
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(2400u64),
            delta: Uint128::from(100u64),
            finders_fee_bps: 0,
            swap_fee_bps: 0,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
    )
    .unwrap();
    println!("pool id is {}", pool.id);

    let token_id_1 = mint(&mut router, &user1, minter);
    approve(&mut router, &user1, &collection, &infinity_pool, token_id_1);
    let token_id_2 = mint(&mut router, &user1, minter);
    approve(&mut router, &user1, &collection, &infinity_pool, token_id_2);
    let msg = ExecuteMsg::DepositNfts {
        pool_id: pool.id,
        collection: collection.to_string(),
        nft_token_ids: vec![token_id_1.to_string(), token_id_2.to_string()],
    };
    let _ = router.execute_contract(user1.clone(), infinity_pool.clone(), &msg, &[]);

    // Only owner of pool can activate pool
    let msg = ExecuteMsg::SetActivePool {
        is_active: false,
        pool_id: pool.id,
    };
    let _ = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);

    // Owner can deposit tokens
    let deposit_amount_1 = 4800u128;
    let msg = ExecuteMsg::DepositTokens { pool_id: pool.id };
    let res = router.execute_contract(
        creator.clone(),
        infinity_pool.clone(),
        &msg,
        &coins(deposit_amount_1, NATIVE_DENOM),
    );
    assert!(res.is_ok());

    let swap_msg = SimDirectSwapNftsForTokens {
        pool_id: pool.id,
        nfts_to_swap: vec![NftSwap {
            nft_token_id: token_id_1.to_string(),
            token_amount: Uint128::new(20),
        }],
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: true,
        },
        token_recipient: _user2.to_string(),
        finder: None,
    };
    (router, infinity_pool, pool, swap_msg, creator)
}

#[test]
fn cant_swap_inactive_pool() {
    let (router, infinity_pool, _, swap_msg, _) = setup_swap_pool();
    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.clone(), &swap_msg);

    let res = res.unwrap_err();
    assert_eq!(
        res,
        StdError::GenericErr {
            msg: "Querier contract error: Generic error: Invalid pool: pool is not active"
                .to_string()
        }
    );
}

#[test]
fn can_swap_active_pool() {
    let (mut router, infinity_pool, pool, swap_msg, creator) = setup_swap_pool();

    let msg = ExecuteMsg::SetActivePool {
        is_active: true,
        pool_id: pool.id,
    };
    let _ = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);

    let res: StdResult<SwapResponse> = router
        .wrap()
        .query_wasm_smart(infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;
    assert_eq!(swaps[0].pool_id, pool.id);
    assert_eq!(swaps[0].spot_price, Uint128::new(2400));
}
