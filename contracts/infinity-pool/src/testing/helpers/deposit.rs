use cosmwasm_std::coins;
use cosmwasm_std::Addr;
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;

use crate::msg::ExecuteMsg;
use crate::state::Pool;
use crate::testing::helpers::nft_functions::approve;
use crate::testing::helpers::nft_functions::mint;
use cw_multi_test::Executor;

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
    println!(
        "minter {:?}, collection {:?}, creator {:?} infnity pool {:?}",
        minter, collection, creator, infinity_pool
    );
    let token_id_1 = mint(router, &creator, &minter);
    approve(router, &creator, &collection, &infinity_pool, token_id_1);
    let token_id_2 = mint(router, &creator, &minter);
    approve(router, &creator, &collection, &infinity_pool, token_id_2);
    let msg = ExecuteMsg::DepositNfts {
        pool_id: pool.id,
        collection: collection.to_string(),
        nft_token_ids: vec![token_id_1.to_string(), token_id_2.to_string()],
    };
    let _ = router.execute_contract(creator, infinity_pool, &msg, &[]);

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
