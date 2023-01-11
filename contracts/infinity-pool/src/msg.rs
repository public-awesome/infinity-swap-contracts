use crate::state::{PoolType, BondingCurve};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
    pub marketplace_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Create a pool for trading a specific collection
    CreatePool {
        collection: String,
        pool_type: PoolType,
        bonding_curve: BondingCurve,
        delta: Uint128,
        fee: Uint128,
        asset_recipient: String,
    },
    UpdatePoolConfig {},
    RemovePool {},
    DepositTokens {},
    DepositNfts {},
    WithdrawTokens {},
    WithdrawNfts {},
    SwapTokenForAnyNfts {},
    SwapTokenForSpecificNfts {},
    SwapNftsForTokens {},
}

#[cw_serde]
pub enum QueryMsg {
}