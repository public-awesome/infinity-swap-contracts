use crate::msg::{ExecuteMsg, PoolsByIdResponse, QueryMsg};
use crate::state::Pool;
use anyhow::Error;
use cosmwasm_std::{coins, Addr, Uint128};
use cw_multi_test::Executor;
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;

pub fn create_pool(
    router: &mut StargazeApp,
    infinity_pool: Addr,
    creator: Addr,
    msg: ExecuteMsg,
) -> Result<Pool, Error> {
    let res = router.execute_contract(creator, infinity_pool.clone(), &msg, &[]);
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
    creator: Addr,
    pool_id: u64,
    deposit_amount: Uint128,
) -> Result<u128, Error> {
    let msg = ExecuteMsg::DepositTokens { pool_id };
    let res = router.execute_contract(
        creator,
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
    creator: Addr,
    pool_id: u64,
    collection: Addr,
    nft_token_ids: Vec<String>,
) -> Result<String, Error> {
    let msg = ExecuteMsg::DepositNfts {
        pool_id,
        collection: collection.to_string(),
        nft_token_ids,
    };
    let res = router.execute_contract(creator, infinity_pool, &msg, &[]);
    assert!(res.is_ok());
    let nft_token_ids = res.unwrap().events[1].attributes[2].value.clone();

    Ok(nft_token_ids)
}

pub fn activate(
    router: &mut StargazeApp,
    infinity_pool: Addr,
    creator: Addr,
    pool_id: u64,
    is_active: bool,
) -> Result<bool, Error> {
    let msg = ExecuteMsg::SetActivePool { pool_id, is_active };
    let res = router.execute_contract(creator, infinity_pool, &msg, &[]);
    assert!(res.is_ok());
    Ok(is_active)
}
