use std::vec;

use crate::msg::QueryMsg::SimDirectSwapNftsForTokens;
use crate::msg::{self, ExecuteMsg};
use crate::msg::{NftSwap, SwapParams};
use crate::state::BondingCurve;
use crate::state::Pool;
use crate::state::PoolType;
use crate::swap_processor::{NftPayment, Swap, TokenPayment};
use crate::testing::helpers::nft_functions::{approve, mint};
use crate::testing::helpers::pool_functions::create_pool;
use crate::testing::setup::msg::MarketAccounts;
use crate::testing::setup::setup_accounts::setup_second_bidder_account;
use crate::testing::setup::setup_infinity_pool::setup_infinity_pool;
use crate::testing::setup::setup_marketplace::setup_marketplace_trading_fee;
use cosmwasm_std::Timestamp;
use cosmwasm_std::{coins, Addr, Uint128};
use cw_multi_test::Executor;
use sg_multi_test::StargazeApp;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::msg::VendingTemplateResponse;
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;

const ASSET_ACCOUNT: &str = "asset";

pub struct SwapPoolResult {
    pub router: StargazeApp,
    pub user1: Addr,
    pub user2: Addr,
    pub creator: Addr,
    pub minter: Addr,
    pub collection: Addr,
    pub infinity_pool: Addr,
    pub pool: Pool,
}

pub fn setup_swap_pool(
    vt: VendingTemplateResponse<MarketAccounts>,
    pool_type: PoolType,
    spot_price: u128,
    trading_fee: Option<u64>,
    finders_fee_bps: Option<u64>,
) -> Result<SwapPoolResult, anyhow::Error> {
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

    let marketplace =
        setup_marketplace_trading_fee(&mut router, creator.clone(), trading_fee).unwrap();
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
            spot_price: Uint128::from(spot_price),
            delta: Uint128::from(100u64),
            finders_fee_bps: finders_fee_bps.unwrap_or(0),
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
pub struct DepositNftsResult {
    pub infinity_pool: Addr,
    pub token_id_1: u32,
    pub token_id_2: u32,
}

pub fn deposit_nfts_and_tokens(
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

pub fn get_sim_swap_message(
    pool: Pool,
    token_id_1: u32,
    token_amount: u128,
    robust: bool,
    user2: Addr,
    finder: Option<String>,
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
        finder,
    }
}

pub fn set_pool_active(
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

pub fn check_nft_sale(
    expected_spot_price: u128,
    expected_royalty_price: u128,
    expected_network_fee: u128,
    expected_finders_fee: u128,
    swaps: Vec<Swap>,
    pool: Pool,
    creator: Addr,
    user2: Addr,
    token_id: String,
) {
    assert_eq!(swaps[0].pool_id, pool.id);
    assert_eq!(swaps[0].spot_price.u128(), expected_spot_price);
    println!("swaps is {:?}", swaps);

    let expected_nft_payment = Some(NftPayment {
        nft_token_id: token_id,
        address: ASSET_ACCOUNT.to_string(),
    });
    assert_eq!(swaps[0].nft_payment, expected_nft_payment);
    let expected_royalty_payment = Some(TokenPayment {
        amount: Uint128::new(expected_royalty_price),
        address: creator.to_string(),
    });
    assert_eq!(swaps[0].royalty_payment, expected_royalty_payment);
    let network_fee = swaps[0].network_fee.u128();
    println!(
        "network fee: {:?} expected_network_fee: {}",
        network_fee, expected_network_fee
    );
    assert_eq!(network_fee, expected_network_fee);
    let mut expected_price = expected_spot_price - network_fee;
    expected_price -= expected_royalty_payment.unwrap().amount.u128();
    println!(
        "expected price {:?} expected finders fee {:?}",
        expected_price, expected_finders_fee
    );
    expected_price -= expected_finders_fee;

    let expected_seller_payment = Some(TokenPayment {
        amount: Uint128::new(expected_price),
        address: user2.to_string(),
    });
    println!("expected seller payment {:?}", expected_seller_payment);
    println!("actual seller payment {:?}", swaps[0].seller_payment);
    assert_eq!(swaps[0].seller_payment, expected_seller_payment);
}
