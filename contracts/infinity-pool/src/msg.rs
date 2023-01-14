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
    CreatePool {
        collection: String,
        asset_recipient: Option<String>,
        pool_type: PoolType,
        bonding_curve: BondingCurve,
        delta: Uint128,
        spot_price: Uint128,
        fee_bps: u16,
    },
    DepositTokens {
        pool_id: u64,
    },
    DepositNfts {
        pool_id: u64,
        collection: String,
        nft_token_ids: Vec<String>,
    },
    WithdrawTokens {
        pool_id: u64,
        amount: Uint128,
        asset_recipient: Option<String>,
    },
    WithdrawAllTokens {
        pool_id: u64,
        asset_recipient: Option<String>,
    },
    WithdrawNfts {
        pool_id: u64,
        nft_token_ids: Vec<String>,
        asset_recipient: Option<String>,
    },
    WithdrawAllNfts {
        pool_id: u64,
        asset_recipient: Option<String>,
    },
    UpdatePoolConfig {
        pool_id: u64,
        asset_recipient: Option<String>,
        delta: Option<Uint128>,
        spot_price: Option<Uint128>,
        fee_bps: Option<u16>,
    },
    SetActivePool {
        is_active: bool,
        pool_id: u64,
    },
    RemovePool {
        pool_id: u64,
        asset_recipient: Option<String>,
    },
    SwapTokenForAnyNfts {
        collection: String,
        num_nfts: u8,
        max_expected_token_input: Uint128,
        asset_recipient: Option<String>,
    },
    SwapTokenForSpecificNfts {},
    SwapNftForTokens {},
}

#[cw_serde]
pub struct QueryOptions<T> {
    pub descending: Option<bool>,
    pub start_after: Option<T>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub enum QueryMsg {
    Pool {
        pool_id: u64,
    },
    Pools {
        descending: Option<bool>,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    PoolsByOwner {
        owner: String,
        query_options: QueryOptions<u64>
    },
    PoolsByBuyPrice {
        descending: Option<bool>,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    PoolsBySellPrice {
        descending: Option<bool>,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}