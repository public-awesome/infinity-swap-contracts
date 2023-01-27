use std::collections::BTreeMap;

use crate::state::{BondingCurve, Config, Pool, PoolQuote, PoolType};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
    pub marketplace_addr: String,
}

#[cw_serde]
pub struct PoolNfts {
    pub pool_id: u64,
    pub nft_token_ids: Vec<String>,
}

#[cw_serde]
pub struct SwapParams {
    pub deadline: Timestamp,
    pub robust: bool,
}

#[cw_serde]
pub struct SwapNft {
    pub nft_token_id: String,
    pub min_expected_token_output: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreatePool {
        collection: String,
        asset_recipient: Option<String>,
        pool_type: PoolType,
        bonding_curve: BondingCurve,
        spot_price: Uint128,
        delta: Uint128,
        fee_bps: Option<u16>,
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
    DirectSwapNftsForTokens {
        pool_id: u64,
        swap_nfts: Vec<SwapNft>,
        swap_params: SwapParams,
        token_recipient: Option<String>,
    },
    SwapNftsForTokens {
        collection: String,
        swap_nfts: Vec<SwapNft>,
        swap_params: SwapParams,
        token_recipient: Option<String>,
    },
    // DirectSwapTokensforSpecificNfts {
    //     pool_id: u64,
    //     swap_nfts: Vec<SwapNft>,
    //     swap_params: SwapParams,
    //     token_recipient: Option<String>,
    // },
    // SwapTokensforSpecificNfts {
    //     pool_id: u64,
    //     swap_nfts: Vec<SwapNft>,
    //     swap_params: SwapParams,
    //     token_recipient: Option<String>,
    // },
    // SwapTokensforAnyNfts {
    //     pool_id: u64,
    //     swap_nfts: Vec<SwapNft>,
    //     swap_params: SwapParams,
    //     token_recipient: Option<String>,
    // },
}

#[cw_serde]
pub struct QueryOptions<T> {
    pub descending: Option<bool>,
    pub start_after: Option<T>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Pools {
        query_options: QueryOptions<u64>,
    },
    PoolsById {
        pool_ids: Vec<u64>,
    },
    PoolsByOwner {
        owner: String,
        query_options: QueryOptions<u64>,
    },
    PoolQuotesBuy {
        collection: String,
        query_options: QueryOptions<(Uint128, u64)>,
    },
    PoolQuotesSell {
        collection: String,
        query_options: QueryOptions<(Uint128, u64)>,
    },
    SimDirectSwapNftsForTokens {
        pool_id: u64,
        swap_nfts: Vec<SwapNft>,
        swap_params: SwapParams,
        token_recipient: String,
    },
    SimSwapNftsForTokens {
        collection: String,
        swap_nfts: Vec<SwapNft>,
        swap_params: SwapParams,
        token_recipient: String,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct PoolsResponse {
    pub pools: Vec<Pool>,
}

#[cw_serde]
pub struct PoolsByIdResponse {
    pub pools: Vec<(u64, Option<Pool>)>,
}

#[cw_serde]
pub struct PoolQuoteResponse {
    pub pool_quotes: Vec<PoolQuote>,
}
