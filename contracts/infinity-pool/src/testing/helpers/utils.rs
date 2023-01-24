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

pub fn assert_error(res: Result<AppResponse, Error>, expected: ContractError) {
    assert_eq!(
        res.unwrap_err().source().unwrap().to_string(),
        expected.to_string()
    );
}
