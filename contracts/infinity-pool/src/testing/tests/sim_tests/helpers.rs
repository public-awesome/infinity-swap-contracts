use std::vec;

use crate::msg::PoolNftSwap;
use crate::msg::QueryMsg::SimDirectSwapNftsForTokens;
use crate::msg::QueryMsg::SimDirectSwapTokensforSpecificNfts;
use crate::msg::QueryMsg::SimSwapNftsForTokens;
use crate::msg::QueryMsg::SimSwapTokensForAnyNfts;
use crate::msg::QueryMsg::SimSwapTokensForSpecificNfts;
use crate::msg::{self, ExecuteMsg};
use crate::msg::{NftSwap, SwapParams};
use crate::state::BondingCurve;
use crate::state::Pool;
use crate::state::PoolType;
use crate::swap_processor::{NftPayment, Swap, TokenPayment};
use crate::testing::helpers::nft_functions::{approve, mint};
use crate::testing::helpers::pool_functions::create_pool;
use crate::testing::setup::setup_infinity_pool::setup_infinity_pool;
use crate::testing::setup::setup_marketplace::setup_marketplace_trading_fee;
use cosmwasm_std::Timestamp;
use cosmwasm_std::{coins, Addr, Uint128};
use cw_multi_test::Executor;
use sg_multi_test::StargazeApp;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;

pub const ASSET_ACCOUNT: &str = "asset";

#[derive(Debug)]
pub struct SwapPoolResult {
    pub user1: Addr,
    pub user2: Addr,
    pub creator: Addr,
    pub minter: Addr,
    pub collection: Addr,
    pub infinity_pool: Addr,
    pub pool: Pool,
}

pub struct SwapPoolSetup {
    pub pool_type: PoolType,
    pub spot_price: u128,
    pub finders_fee_bps: Option<u64>,
}

pub struct VendingTemplateSetup<'a> {
    pub router: &'a mut StargazeApp,
    pub minter: &'a Addr,
    pub collection: &'a Addr,
    pub creator: Addr,
    pub user1: Addr,
    pub user2: Addr,
}

pub struct NftSaleCheckParams {
    pub expected_spot_price: u128,
    pub expected_royalty_price: u128,
    pub expected_network_fee: u128,
    pub expected_finders_fee: u128,
    pub swaps: Vec<Swap>,
    pub creator: Addr,
    pub expected_seller: Addr,
    pub token_id: String,
    pub expected_nft_payer: Addr,
    pub expected_finder: Addr,
}

pub fn setup_swap_pool(
    vts: VendingTemplateSetup,
    swap_pool_configs: Vec<SwapPoolSetup>,
    trading_fee: Option<u64>,
) -> Vec<Result<SwapPoolResult, anyhow::Error>> {
    let (router, minter, creator, user1, user2) =
        (vts.router, vts.minter, vts.creator, vts.user1, vts.user2);
    let minter = minter.to_owned();

    let collection = vts.collection;
    let asset_account = Addr::unchecked(ASSET_ACCOUNT);

    let marketplace = setup_marketplace_trading_fee(router, creator.clone(), trading_fee).unwrap();

    setup_block_time(router, GENESIS_MINT_START_TIME, None);
    let infinity_pool = setup_infinity_pool(router, creator.clone(), marketplace).unwrap();

    let mut results: Vec<Result<SwapPoolResult, anyhow::Error>> = vec![];

    for swap_pool_config in swap_pool_configs {
        let infinity_pool = infinity_pool.clone();
        let creator = creator.clone();
        let user1 = user1.clone();
        let user2 = user2.clone();
        let minter = minter.clone();
        let collection = collection.clone();

        // Can create a Linear Nft Pool
        let pool_result = create_pool(
            router,
            infinity_pool.clone(),
            creator.clone(),
            ExecuteMsg::CreatePool {
                collection: collection.to_string(),
                asset_recipient: Some(asset_account.to_string()),
                pool_type: swap_pool_config.pool_type,
                bonding_curve: BondingCurve::Linear,
                spot_price: Uint128::from(swap_pool_config.spot_price),
                delta: Uint128::from(100u64),
                finders_fee_bps: swap_pool_config.finders_fee_bps.unwrap_or(0),
                swap_fee_bps: 0,
                reinvest_tokens: false,
                reinvest_nfts: false,
            },
        );
        let result = match pool_result {
            Ok(result) => Ok(SwapPoolResult {
                user1,
                user2,
                creator,
                minter,
                collection,
                infinity_pool,
                pool: result,
            }),
            Err(err) => Err(err),
        };
        results.push(result);
    }
    results
}
pub struct DepositNftsResult {
    pub token_id_1: u32,
    pub token_id_2: u32,
}

pub fn deposit_nfts_and_tokens(
    router: &mut StargazeApp,
    minter: Addr,
    collection: Addr,
    infinity_pool: Addr,
    pool: Pool,
    creator: Addr,
    deposit_amount: u128,
) -> DepositNftsResult {
    let tokens = deposit_nfts(
        router,
        minter,
        collection,
        infinity_pool.clone(),
        pool.clone(),
        creator.clone(),
    );
    let _ = deposit_tokens(router, deposit_amount, infinity_pool, pool, creator);
    DepositNftsResult {
        token_id_1: tokens.token_id_1,
        token_id_2: tokens.token_id_2,
    }
}

pub fn deposit_nfts(
    router: &mut StargazeApp,
    minter: Addr,
    collection: Addr,
    infinity_pool: Addr,
    pool: Pool,
    creator: Addr,
) -> DepositNftsResult {
    let token_id_1 = mint(router, &creator, &minter);
    approve(router, &creator, &collection, &infinity_pool, token_id_1);
    let token_id_2 = mint(router, &creator, &minter);
    approve(router, &creator, &collection, &infinity_pool, token_id_2);
    let msg = ExecuteMsg::DepositNfts {
        pool_id: pool.id,
        collection: collection.to_string(),
        nft_token_ids: vec![token_id_1.to_string(), token_id_2.to_string()],
    };
    let res = router.execute_contract(creator, infinity_pool, &msg, &[]);

    DepositNftsResult {
        token_id_1,
        token_id_2,
    }
}

pub fn deposit_one_nft(
    router: &mut StargazeApp,
    minter: Addr,
    collection: Addr,
    infinity_pool: Addr,
    pool: Pool,
    creator: Addr,
) -> u32 {
    let token_id_1 = mint(router, &creator, &minter);
    approve(router, &creator, &collection, &infinity_pool, token_id_1);

    let msg = ExecuteMsg::DepositNfts {
        pool_id: pool.id,
        collection: collection.to_string(),
        nft_token_ids: vec![token_id_1.to_string()],
    };
    let _ = router.execute_contract(creator, infinity_pool, &msg, &[]);
    token_id_1
}

pub fn deposit_tokens(
    router: &mut StargazeApp,
    deposit_amount: u128,
    infinity_pool: Addr,
    pool: Pool,
    creator: Addr,
) -> Result<(), anyhow::Error> {
    // Owner can deposit tokens
    let deposit_amount_1 = deposit_amount;
    let msg = ExecuteMsg::DepositTokens { pool_id: pool.id };
    let res = router.execute_contract(
        creator,
        infinity_pool,
        &msg,
        &coins(deposit_amount_1, NATIVE_DENOM),
    );
    match res {
        Ok(_) => Ok(()),
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
        sender: user2.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust,
            asset_recipient: None,
            finder,
        },
    }
}

pub fn get_sim_swap_nfts_for_tokens_msg(
    collection: &Addr,
    token_id_1: u32,
    token_amount: u128,
    robust: bool,
    user2: Addr,
    finder: Option<String>,
) -> msg::QueryMsg {
    SimSwapNftsForTokens {
        collection: collection.to_string(),
        nfts_to_swap: vec![NftSwap {
            nft_token_id: token_id_1.to_string(),
            token_amount: Uint128::new(token_amount),
        }],
        sender: user2.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust,
            asset_recipient: None,
            finder,
        },
    }
}

pub fn get_sim_direct_swap_tokens_for_specific_nfts_msg(
    pool: Pool,
    token_id_1: u32,
    token_amount: u128,
    robust: bool,
    user2: Addr,
    finder: Option<String>,
) -> msg::QueryMsg {
    SimDirectSwapTokensforSpecificNfts {
        pool_id: pool.id,
        nfts_to_swap_for: vec![NftSwap {
            nft_token_id: token_id_1.to_string(),
            token_amount: Uint128::new(token_amount),
        }],
        sender: user2.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust,
            asset_recipient: None,
            finder,
        },
    }
}

pub fn get_swap_tokens_for_specific_nfts_msg(
    pool: Pool,
    collection: Addr,
    token_id_1: u32,
    token_amount: u128,
    robust: bool,
    user2: Addr,
    finder: Option<String>,
) -> msg::QueryMsg {
    SimSwapTokensForSpecificNfts {
        pool_nfts_to_swap_for: vec![PoolNftSwap {
            pool_id: pool.id,
            nft_swaps: vec![NftSwap {
                nft_token_id: token_id_1.to_string(),
                token_amount: Uint128::new(token_amount),
            }],
        }],
        sender: user2.to_string(),
        collection: collection.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust,
            asset_recipient: None,
            finder,
        },
    }
}

pub fn get_swap_tokens_for_any_nfts_msg(
    collection: Addr,
    max_expected_token_input: Vec<Uint128>,
    robust: bool,
    user2: Addr,
    finder: Option<String>,
) -> msg::QueryMsg {
    SimSwapTokensForAnyNfts {
        sender: user2.to_string(),
        collection: collection.to_string(),
        max_expected_token_input,
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust,
            asset_recipient: None,
            finder,
        },
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

pub fn check_nft_sale(scp: NftSaleCheckParams) {
    assert_eq!(scp.swaps[0].spot_price.u128(), scp.expected_spot_price);
    let expected_nft_payment = Some(NftPayment {
        nft_token_id: scp.token_id,
        address: scp.expected_nft_payer.to_string(),
    });
    // assert_eq!(scp.swaps[0].nft_payment, expected_nft_payment);

    let network_fee = scp.swaps[0].network_fee.u128();

    assert_eq!(network_fee, scp.expected_network_fee);
    let mut expected_price = scp.expected_spot_price - network_fee;
    expected_price -= scp.expected_royalty_price;
    expected_price -= scp.expected_finders_fee;

    let expected_seller_payment = Some(TokenPayment {
        amount: Uint128::new(expected_price),
        address: scp.expected_seller.to_string(),
    });
    assert_eq!(scp.swaps[0].seller_payment, expected_seller_payment);

    let expected_finder_payment = match scp.expected_finders_fee {
        0 => None,
        _ => Some(TokenPayment {
            amount: Uint128::from(scp.expected_finders_fee),
            address: scp.expected_finder.to_string(),
        }),
    };
    assert_eq!(scp.swaps[0].finder_payment, expected_finder_payment);

    let expected_royalty_payment = Some(TokenPayment {
        amount: Uint128::new(scp.expected_royalty_price),
        address: scp.creator.to_string(),
    });
    assert_eq!(scp.swaps[0].royalty_payment, expected_royalty_payment);
}
