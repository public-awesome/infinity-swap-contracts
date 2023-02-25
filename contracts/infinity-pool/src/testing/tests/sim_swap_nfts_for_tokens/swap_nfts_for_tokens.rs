use std::vec;

use crate::msg::QueryMsg::SimDirectSwapNftsForTokens;
use crate::msg::SwapResponse;
use crate::msg::{self, ExecuteMsg};
use crate::msg::{NftSwap, SwapParams};
use crate::state::BondingCurve;
use crate::state::Pool;
use crate::state::PoolType;
use crate::swap_processor::{NftPayment, Swap, TokenPayment};
use crate::testing::helpers::nft_functions::{approve, mint};
use crate::testing::helpers::pool_functions::create_pool;
use crate::testing::setup::setup_accounts::setup_second_bidder_account;
use crate::testing::setup::setup_infinity_pool::setup_infinity_pool;
use crate::testing::setup::setup_marketplace::setup_marketplace;
use crate::testing::setup::templates::standard_minter_template;
use cosmwasm_std::StdError;
use cosmwasm_std::StdError::GenericErr;
use cosmwasm_std::StdResult;
use cosmwasm_std::Timestamp;
use cosmwasm_std::{coins, Addr, Uint128};
use cw_multi_test::Executor;
use sg_multi_test::StargazeApp;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;

const ASSET_ACCOUNT: &str = "asset";

struct SwapPoolResult {
    router: StargazeApp,
    user1: Addr,
    user2: Addr,
    creator: Addr,
    minter: Addr,
    collection: Addr,
    infinity_pool: Addr,
    pool: Pool,
}

fn setup_swap_pool(pool_type: PoolType) -> Result<SwapPoolResult, anyhow::Error> {
    let vt = standard_minter_template(5000);
    let (mut router, minter, creator, user1) = (
        vt.router,
        vt.collection_response_vec[0].minter.as_ref().unwrap(),
        vt.accts.creator,
        vt.accts.bidder,
    );
    let minter = minter.to_owned();
    let collection = vt.collection_response_vec[0].collection.clone().unwrap();
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);
    let user2 = setup_second_bidder_account(&mut router).unwrap();

    let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
    let infinity_pool = setup_infinity_pool(&mut router, creator.clone(), marketplace).unwrap();

    setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);
    // Can create a Linear Nft Pool
    let pool_result = create_pool(
        &mut router,
        infinity_pool.clone(),
        creator.clone(),
        ExecuteMsg::CreatePool {
            collection: collection.to_string(),
            asset_recipient: Some(asset_account.to_string()),
            pool_type,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(1000u64),
            delta: Uint128::from(100u64),
            finders_fee_bps: 0,
            swap_fee_bps: 0,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
    );

    match pool_result {
        Ok(result) => Ok(SwapPoolResult {
            router,
            user1,
            user2,
            creator,
            minter,
            collection,
            infinity_pool,
            pool: result,
        }),
        Err(err) => Err(err),
    }
}
struct DepositNftsResult {
    infinity_pool: Addr,
    token_id_1: u32,
    token_id_2: u32,
}

fn deposit_nfts_and_tokens(
    router: &mut StargazeApp,
    user1: Addr,
    deposit_amount: u128,
    minter: Addr,
    collection: Addr,
    infinity_pool: Addr,
    pool: Pool,
    creator: Addr,
) -> Result<DepositNftsResult, anyhow::Error> {
    let token_id_1 = mint(router, &user1, &minter);
    approve(router, &user1, &collection, &infinity_pool, token_id_1);
    let token_id_2 = mint(router, &user1, &minter);
    approve(router, &user1, &collection, &infinity_pool, token_id_2);
    let msg = ExecuteMsg::DepositNfts {
        pool_id: pool.id,
        collection: collection.to_string(),
        nft_token_ids: vec![token_id_1.to_string(), token_id_2.to_string()],
    };
    let _ = router.execute_contract(user1.clone(), infinity_pool.clone(), &msg, &[]);

    // Owner can deposit tokens
    let deposit_amount_1 = deposit_amount;
    let msg = ExecuteMsg::DepositTokens { pool_id: pool.id };
    let res = router.execute_contract(
        creator,
        infinity_pool.clone(),
        &msg,
        &coins(deposit_amount_1, NATIVE_DENOM),
    );
    match res {
        Ok(_) => Ok(DepositNftsResult {
            infinity_pool,
            token_id_1,
            token_id_2,
        }),
        Err(err) => Err(err),
    }
}

fn get_sim_swap_message(
    pool: Pool,
    token_id_1: u32,
    token_amount: u128,
    robust: bool,
    user2: Addr,
) -> msg::QueryMsg {
    SimDirectSwapNftsForTokens {
        pool_id: pool.id,
        nfts_to_swap: vec![NftSwap {
            nft_token_id: token_id_1.to_string(),
            token_amount: Uint128::new(token_amount),
        }],
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust,
        },
        token_recipient: user2.to_string(),
        finder: None,
    }
}

fn set_pool_active(
    router: &mut StargazeApp,
    is_active: bool,
    pool: Pool,
    creator: Addr,
    infinity_pool: Addr,
) {
    let msg = ExecuteMsg::SetActivePool {
        is_active,
        pool_id: pool.id,
    };
    let _ = router.execute_contract(creator, infinity_pool, &msg, &[]);
}

fn check_nft_sale(
    sale_price: u128,
    royalty_price: u128,
    process_royalty: bool,
    swaps: Vec<Swap>,
    pool: Pool,
    creator: Addr,
    user2: Addr,
    token_id: String,
) {
    assert_eq!(swaps[0].pool_id, pool.id);
    assert_eq!(swaps[0].spot_price, Uint128::new(1000));
    println!("swaps is {:?}", swaps);

    let expected_nft_payment = Some(NftPayment {
        nft_token_id: token_id,
        address: ASSET_ACCOUNT.to_string(),
    });
    assert_eq!(swaps[0].nft_payment, expected_nft_payment);
    let expected_royalty_payment = Some(TokenPayment {
        amount: Uint128::new(royalty_price),
        address: creator.to_string(),
    });
    assert_eq!(swaps[0].royalty_payment, expected_royalty_payment);
    let network_fee = swaps[0].network_fee;
    let mut expected_price = sale_price - network_fee.u128();
    if process_royalty {
        expected_price -= expected_royalty_payment.unwrap().amount.u128();
    }
    let expected_seller_payment = Some(TokenPayment {
        amount: Uint128::new(expected_price),
        address: user2.to_string(),
    });
    assert_eq!(swaps[0].seller_payment, expected_seller_payment);
}
#[test]
fn cant_swap_inactive_pool() {
    let mut spr: SwapPoolResult = setup_swap_pool(PoolType::Token).unwrap();
    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut spr.router,
        spr.user1,
        1000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    set_pool_active(
        &mut spr.router,
        false,
        spr.pool.clone(),
        spr.creator,
        spr.infinity_pool,
    );

    let swap_msg = get_sim_swap_message(spr.pool, dnr.token_id_1, 1000, true, spr.user2);
    let res: StdResult<SwapResponse> = spr
        .router
        .wrap()
        .query_wasm_smart(dnr.infinity_pool, &swap_msg);

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
    let mut spr: SwapPoolResult = setup_swap_pool(PoolType::Token).unwrap();
    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut spr.router,
        spr.user1,
        1000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    let (sale_price, royalty_price) = (1000_u128, 100_u128);
    let swap_msg = get_sim_swap_message(
        spr.pool.clone(),
        dnr.token_id_1,
        sale_price,
        true,
        spr.user2.clone(),
    );

    set_pool_active(
        &mut spr.router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let res: StdResult<SwapResponse> = spr
        .router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;
    check_nft_sale(
        sale_price,
        royalty_price,
        true,
        swaps,
        spr.pool,
        spr.creator,
        spr.user2,
        dnr.token_id_1.to_string(),
    )
}

#[test]
fn invalid_nft_pool_can_not_deposit() {
    let mut spr: SwapPoolResult = setup_swap_pool(PoolType::Nft).unwrap();
    let dnr = deposit_nfts_and_tokens(
        &mut spr.router,
        spr.user1,
        1000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    );
    let error_res = dnr.err().unwrap();
    assert_eq!(
        error_res.root_cause().to_string(),
        "Invalid pool: cannot deposit tokens into nft pool"
    );
}

#[test]
fn not_enough_deposit_no_swap() {
    let mut spr: SwapPoolResult = setup_swap_pool(PoolType::Token).unwrap();
    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut spr.router,
        spr.user1,
        500_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    set_pool_active(
        &mut spr.router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );
    let (sale_price, royalty_price) = (1000_u128, 100_u128);
    let swap_msg = get_sim_swap_message(
        spr.pool.clone(),
        dnr.token_id_1,
        sale_price,
        false,
        spr.user2.clone(),
    );

    let res: StdResult<SwapResponse> = spr
        .router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    let error_msg = res.err().unwrap();

    let expected_error = GenericErr {
        msg: "Querier contract error: \
        Generic error: Swap error: pool cannot offer quote"
            .to_string(),
    };
    assert_eq!(error_msg, expected_error);

    let msg = ExecuteMsg::DepositTokens {
        pool_id: spr.pool.id,
    };
    let _ = spr.router.execute_contract(
        spr.creator.clone(),
        spr.infinity_pool.clone(),
        &msg,
        &coins(1000_u128, NATIVE_DENOM),
    );

    // swap wil be valid now there there is sufficent payment
    let res: StdResult<SwapResponse> = spr
        .router
        .wrap()
        .query_wasm_smart(spr.infinity_pool.clone(), &swap_msg);
    let swaps = res.unwrap().swaps;
    check_nft_sale(
        sale_price,
        royalty_price,
        true,
        swaps,
        spr.pool,
        spr.creator,
        spr.user2,
        dnr.token_id_1.to_string(),
    )
}

#[test]
fn invalid_sale_price_below_min_expected() {
    let mut spr: SwapPoolResult = setup_swap_pool(PoolType::Token).unwrap();
    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut spr.router,
        spr.user1,
        1000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    let swap_msg = get_sim_swap_message(spr.pool.clone(), dnr.token_id_1, 1200, false, spr.user2);

    set_pool_active(
        &mut spr.router,
        true,
        spr.pool.clone(),
        spr.creator,
        spr.infinity_pool.clone(),
    );

    let error_res: StdResult<SwapResponse> = spr
        .router
        .wrap()
        .query_wasm_smart(spr.infinity_pool, &swap_msg);
    let error_msg = error_res.err().unwrap();

    let expected_error = GenericErr {
        msg: "Querier contract error: Generic error: Swap error: \
        pool sale price is below min expected"
            .to_string(),
    };
    assert_eq!(error_msg, expected_error);
}

#[test]
fn robust_query_does_not_revert_whole_tx_on_error() {
    let mut spr: SwapPoolResult = setup_swap_pool(PoolType::Token).unwrap();
    let dnr: DepositNftsResult = deposit_nfts_and_tokens(
        &mut spr.router,
        spr.user1,
        2000_u128,
        spr.minter,
        spr.collection,
        spr.infinity_pool.clone(),
        spr.pool.clone(),
        spr.creator.clone(),
    )
    .unwrap();
    let sale_price_too_high = 1200_u128;
    let royalty_price = 100_u128;
    let sale_price_valid = 900_u128;
    let swap_msg = SimDirectSwapNftsForTokens {
        pool_id: spr.pool.id,
        nfts_to_swap: vec![
            NftSwap {
                nft_token_id: dnr.token_id_2.to_string(),
                token_amount: Uint128::new(sale_price_valid),
            },
            NftSwap {
                nft_token_id: dnr.token_id_1.to_string(),
                token_amount: Uint128::new(sale_price_too_high), // won't swap bc price too high
            },
        ],
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust: true,
        },
        token_recipient: spr.user2.to_string(),
        finder: None,
    };

    set_pool_active(
        &mut spr.router,
        true,
        spr.pool.clone(),
        spr.creator.clone(),
        spr.infinity_pool.clone(),
    );

    let res: StdResult<SwapResponse> = spr
        .router
        .wrap()
        .query_wasm_smart(spr.infinity_pool, &swap_msg);
    assert!(res.is_ok());
    let swaps = res.unwrap().swaps;
    check_nft_sale(
        sale_price_valid,
        royalty_price,
        false,
        swaps,
        spr.pool,
        spr.creator,
        spr.user2,
        dnr.token_id_2.to_string(),
    )
}
