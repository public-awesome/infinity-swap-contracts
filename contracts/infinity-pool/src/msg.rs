use crate::state::{PoolType};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128};

#[cw_serde]
pub struct InstantiateMsg {
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Create a pool for trading a specific collection
    CreatePool {
        collection: String,
        pool_type: PoolType,
        delta: Uint128,
        fee: Uint128,
        asset_recipient: String,
    },
}