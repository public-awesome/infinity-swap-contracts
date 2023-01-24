use std::fmt::DebugList;

use crate::helpers::{get_next_pool_counter, save_pool};
use crate::msg::ExecuteMsg;
use crate::state::{BondingCurve, Pool, PoolType};
use crate::ContractError;
use anyhow::Error;
use cosmwasm_std::{Addr, Storage, Uint128};
use cw_multi_test::Executor;
use cw_multi_test::{
    App, AppResponse, BankKeeper, BasicAppBuilder, CosmosRouter, Module, WasmKeeper,
};
use sg_multi_test::StargazeApp;

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
