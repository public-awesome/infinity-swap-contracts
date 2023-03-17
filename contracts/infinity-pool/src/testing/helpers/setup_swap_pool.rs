use std::vec;

use crate::msg::ExecuteMsg;
use crate::state::BondingCurve;
use crate::state::Pool;
use crate::state::PoolType;
use crate::testing::helpers::deposit::deposit_one_nft;
use crate::testing::helpers::deposit::deposit_tokens;
use crate::testing::helpers::msg::ProcessSwapPoolResultsResponse;
use crate::testing::helpers::msg::SwapPoolResult;
use crate::testing::helpers::msg::SwapPoolSetup;
use crate::testing::helpers::msg::VendingTemplateSetup;
use crate::testing::helpers::pool_functions::create_pool;
use crate::testing::setup::setup_infinity_pool::setup_infinity_pool;
use crate::testing::setup::setup_marketplace::setup_marketplace_trading_fee;
use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::Executor;
use sg_multi_test::StargazeApp;
use sg_std::GENESIS_MINT_START_TIME;
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;

pub const ASSET_ACCOUNT: &str = "asset";

pub fn setup_swap_pool(
    router: &mut StargazeApp,
    vts: VendingTemplateSetup,
    swap_pool_configs: Vec<SwapPoolSetup>,
    trading_fee: Option<u64>,
) -> Vec<Result<SwapPoolResult, anyhow::Error>> {
    let (minter, creator, user1, user2) = (vts.minter, vts.creator, vts.user1, vts.user2);
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

        let create_pool_msg = match swap_pool_config.pool_type {
            PoolType::Token => ExecuteMsg::CreateTokenPool {
                collection: collection.to_string(),
                asset_recipient: Some(asset_account.to_string()),
                bonding_curve: BondingCurve::Linear,
                spot_price: Uint128::from(swap_pool_config.spot_price),
                delta: Uint128::from(100u64),
                finders_fee_bps: swap_pool_config.finders_fee_bps.unwrap_or(0),
            },
            PoolType::Nft => ExecuteMsg::CreateNftPool {
                collection: collection.to_string(),
                asset_recipient: Some(asset_account.to_string()),
                bonding_curve: BondingCurve::Linear,
                spot_price: Uint128::from(swap_pool_config.spot_price),
                delta: Uint128::from(100u64),
                finders_fee_bps: swap_pool_config.finders_fee_bps.unwrap_or(0),
            },
            PoolType::Trade => ExecuteMsg::CreateTradePool {
                collection: collection.to_string(),
                asset_recipient: Some(asset_account.to_string()),
                bonding_curve: BondingCurve::Linear,
                spot_price: Uint128::from(swap_pool_config.spot_price),
                delta: Uint128::from(100u64),
                finders_fee_bps: swap_pool_config.finders_fee_bps.unwrap_or(0),
                swap_fee_bps: 0u64,
                reinvest_tokens: false,
                reinvest_nfts: false,
            },
        };

        // Can create a Linear Nft Pool
        let pool_result = create_pool(
            router,
            infinity_pool.clone(),
            creator.clone(),
            create_pool_msg,
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

fn process_swap_result(
    router: &mut StargazeApp,
    swap_result: &Result<SwapPoolResult, anyhow::Error>,
    deposit_amount: u128,
    skip_deposit_nfts: bool,
) -> (u32, Addr, Addr, Addr, Pool, Addr, Addr, Addr) {
    let r = swap_result.as_ref().unwrap();

    set_pool_active(
        router,
        true,
        r.pool.clone(),
        r.creator.clone(),
        r.infinity_pool.clone(),
    );
    let minter = r.minter.clone();
    let collection = r.collection.clone();
    let infinity_pool = r.infinity_pool.clone();
    let pool = r.pool.clone();
    let creator = r.creator.clone();
    let user1 = r.user1.clone();
    let user2 = r.user2.clone();
    let token_id = match skip_deposit_nfts {
        false => deposit_one_nft(
            router,
            minter.clone(),
            collection.clone(),
            infinity_pool.clone(),
            pool.clone(),
            creator.clone(),
        ),
        true => 0,
    };
    let _ = deposit_tokens(
        router,
        deposit_amount,
        r.infinity_pool.clone(),
        r.pool.clone(),
        r.creator.clone(),
    );

    (
        token_id,
        minter,
        collection,
        infinity_pool,
        pool,
        creator,
        user1,
        user2,
    )
}
pub fn execute_process_swap_results(
    router: &mut StargazeApp,
    swap_results: Vec<Result<SwapPoolResult, anyhow::Error>>,
    deposit_amounts: Vec<u128>,
    skip_deposit_nfts: bool,
) -> Vec<u32> {
    let mut token_ids = vec![];
    for (i, result) in swap_results.iter().enumerate() {
        let token_id = process_swap_result(router, result, deposit_amounts[i], skip_deposit_nfts).0;
        token_ids.append(&mut vec![token_id]);
    }
    token_ids.reverse();
    token_ids
}

pub fn process_swap_results(
    router: &mut StargazeApp,
    vts: VendingTemplateSetup,
    swap_pool_configs: Vec<SwapPoolSetup>,
    deposit_amounts: Vec<u128>,
    trading_fee: Option<u64>,
    skip_deposit_nfts: Option<bool>,
) -> ProcessSwapPoolResultsResponse {
    let mut swap_results: Vec<Result<SwapPoolResult, anyhow::Error>> =
        setup_swap_pool(router, vts, swap_pool_configs, trading_fee);
    let swap_result = swap_results.pop().unwrap();
    let (token_id, minter, collection, infinity_pool, pool, creator, user1, user2) =
        process_swap_result(
            router,
            &swap_result,
            deposit_amounts[0],
            skip_deposit_nfts.unwrap_or(false),
        );
    let mut token_ids_2 = execute_process_swap_results(
        router,
        swap_results,
        deposit_amounts[1..].into(),
        skip_deposit_nfts.unwrap_or(false),
    );

    let mut token_ids = vec![token_id];
    token_ids.append(&mut token_ids_2);
    ProcessSwapPoolResultsResponse {
        minter,
        collection,
        infinity_pool,
        pool,
        creator,
        user1,
        user2,
        token_ids,
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
