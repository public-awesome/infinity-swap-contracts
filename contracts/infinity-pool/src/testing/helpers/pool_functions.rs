use crate::msg::{ExecuteMsg as InfinityPoolExecuteMsg, PoolsByIdResponse, QueryMsg};
use crate::state::Pool;
use crate::testing::setup::setup_marketplace::LISTING_FEE;
use anyhow::Error;
use cosmwasm_std::{coins, Addr, Uint128};
use cw_multi_test::Executor;
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;

use super::fixtures::get_pool_fixtures;

pub fn create_pool(
    router: &mut StargazeApp,
    infinity_pool: Addr,
    owner: Addr,
    msg: InfinityPoolExecuteMsg,
) -> Result<Pool, Error> {
    let res = router.execute_contract(
        owner,
        infinity_pool.clone(),
        &msg,
        &coins(LISTING_FEE, NATIVE_DENOM),
    );
    assert!(res.is_ok());
    let pool_id = res.unwrap().events[1].attributes[1]
        .value
        .parse::<u64>()
        .unwrap();

    let query_msg = QueryMsg::PoolsById {
        pool_ids: vec![pool_id],
    };
    let res: PoolsByIdResponse = router
        .wrap()
        .query_wasm_smart(infinity_pool, &query_msg)
        .unwrap();

    let pool = res.pools[0].1.clone().unwrap();
    Ok(pool)
}

pub fn deposit_tokens(
    router: &mut StargazeApp,
    infinity_pool: Addr,
    owner: Addr,
    pool_id: u64,
    deposit_amount: Uint128,
) -> Result<u128, Error> {
    let msg = InfinityPoolExecuteMsg::DepositTokens { pool_id };
    let res = router.execute_contract(
        owner,
        infinity_pool,
        &msg,
        &coins(deposit_amount.u128(), NATIVE_DENOM),
    );
    assert!(res.is_ok());
    let total_tokens = res.unwrap().events[1].attributes[3]
        .value
        .parse::<u128>()
        .unwrap();

    Ok(total_tokens)
}

pub fn deposit_nfts(
    router: &mut StargazeApp,
    infinity_pool: Addr,
    owner: Addr,
    pool_id: u64,
    collection: Addr,
    nft_token_ids: Vec<String>,
) -> Result<String, Error> {
    let msg = InfinityPoolExecuteMsg::DepositNfts {
        pool_id,
        collection: collection.to_string(),
        nft_token_ids,
    };
    let res = router.execute_contract(owner, infinity_pool, &msg, &[]);
    assert!(res.is_ok());
    let nft_token_ids = res.unwrap().events[1].attributes[2].value.clone();

    Ok(nft_token_ids)
}

pub fn activate(
    router: &mut StargazeApp,
    infinity_pool: &Addr,
    owner: &Addr,
    pool_id: u64,
    is_active: bool,
) -> Result<Pool, Error> {
    let msg = InfinityPoolExecuteMsg::SetActivePool { pool_id, is_active };
    let res = router.execute_contract(owner.clone(), infinity_pool.clone(), &msg, &[]);
    assert!(res.is_ok());
    let query_msg = QueryMsg::PoolsById {
        pool_ids: vec![pool_id],
    };
    let res: PoolsByIdResponse = router
        .wrap()
        .query_wasm_smart(infinity_pool, &query_msg)
        .unwrap();

    let pool = res.pools[0].1.clone().unwrap();
    Ok(pool)
}

pub fn prepare_swap_pool(
    router: &mut StargazeApp,
    infinity_pool: &Addr,
    owner: &Addr,
    num_deposit_tokens: Uint128,
    nft_token_ids: Vec<String>,
    is_active: bool,
    create_pool_msg: InfinityPoolExecuteMsg,
) -> Result<Pool, Error> {
    let pool = create_pool(
        router,
        infinity_pool.clone(),
        owner.clone(),
        create_pool_msg,
    )?;

    if num_deposit_tokens > Uint128::zero() {
        deposit_tokens(
            router,
            infinity_pool.clone(),
            owner.clone(),
            pool.id,
            num_deposit_tokens,
        )?;
    }

    if nft_token_ids.len() > 0 {
        deposit_nfts(
            router,
            infinity_pool.clone(),
            owner.clone(),
            pool.id,
            pool.collection.clone(),
            nft_token_ids,
        )?;
    }

    let pool = activate(router, infinity_pool, owner, pool.id, is_active)?;

    Ok(pool)
}

pub fn prepare_pool_variations(
    router: &mut StargazeApp,
    num_pools: u8,
    asset_account: &Option<String>,
    infinity_pool: &Addr,
    collection: &Addr,
    owner: &Addr,
    deposit_tokens_per_pool: Uint128,
    nft_token_ids: Vec<String>,
    nfts_per_pool: u8,
    is_active: bool,
    finders_fee_bps: u64,
    swap_fee_bps: u64,
    reinvest: bool,
) -> Vec<Pool> {
    let mut nft_token_ids = nft_token_ids;
    let mut pools: Vec<Pool> = vec![];
    let pool_fixtures = &get_pool_fixtures(
        collection,
        asset_account,
        finders_fee_bps,
        swap_fee_bps,
        reinvest,
    )[0..(num_pools as usize)];

    for fixt in pool_fixtures {
        let (curr_deposit_tokens_per_pool, curr_nft_token_ids) = match fixt {
            InfinityPoolExecuteMsg::CreateTokenPool { .. } => (deposit_tokens_per_pool, vec![]),
            InfinityPoolExecuteMsg::CreateNftPool { .. } => {
                let nft_token_ids_slice: Vec<String> =
                    nft_token_ids.drain(0..(nfts_per_pool as usize)).collect();
                (Uint128::zero(), nft_token_ids_slice)
            }
            InfinityPoolExecuteMsg::CreateTradePool { .. } => {
                let nft_token_ids_slice: Vec<String> =
                    nft_token_ids.drain(0..(nfts_per_pool as usize)).collect();
                (deposit_tokens_per_pool, nft_token_ids_slice)
            }
            _ => panic!("Invalid pool fixture"),
        };

        let pool = prepare_swap_pool(
            router,
            infinity_pool,
            owner,
            curr_deposit_tokens_per_pool,
            curr_nft_token_ids.clone(),
            is_active,
            fixt.clone(),
        )
        .unwrap();
        pools.push(pool);
    }
    pools
}
