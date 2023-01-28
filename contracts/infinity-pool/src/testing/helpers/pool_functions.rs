use crate::msg::ExecuteMsg;
use crate::state::{BondingCurve, PoolType};
use anyhow::Error;
use cosmwasm_std::{coins, Addr, Uint128};
use cw_multi_test::Executor;
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;

pub fn create_pool(
    router: &mut StargazeApp,
    infinity_pool: Addr,
    creator: Addr,
    collection: Addr,
    asset_account: Option<Addr>,
    pool_type: PoolType,
    bonding_curve: BondingCurve,
    spot_price: Uint128,
    delta: Uint128,
    fee_bps: Option<u16>,
) -> Result<u64, Error> {
    let msg = ExecuteMsg::CreatePool {
        collection: collection.to_string(),
        asset_recipient: asset_account.map(|a| a.to_string()),
        pool_type,
        bonding_curve,
        spot_price,
        delta,
        fee_bps,
    };
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert!(res.is_ok());
    let pool_id = res.unwrap().events[1].attributes[1]
        .value
        .parse::<u64>()
        .unwrap();
    Ok(pool_id)
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
        creator.clone(),
        infinity_pool.clone(),
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
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
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
    let res = router.execute_contract(creator.clone(), infinity_pool.clone(), &msg, &[]);
    assert!(res.is_ok());
    Ok(is_active)
}
