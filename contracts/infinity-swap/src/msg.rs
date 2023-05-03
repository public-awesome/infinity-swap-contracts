use crate::state::{BondingCurve, Config, Pool, PoolQuote};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use infinity_macros::{infinity_module_execute, infinity_module_query};
use infinity_shared::interface::{NftOrder, SwapParams, SwapResponse};

#[cw_serde]
pub struct InstantiateMsg {
    /// The fungible token used in the pools
    pub denom: String,
    /// The address of the marketplace contract
    pub marketplace_addr: String,
    /// The address of the developer who will receive a portion of the fair burn
    pub developer: Option<String>,
}

#[infinity_module_execute]
#[cw_serde]
pub enum ExecuteMsg {
    /// Create a new pool, defaults to an inactive state
    CreateTokenPool {
        collection: String,
        asset_recipient: Option<String>,
        bonding_curve: BondingCurve,
        spot_price: Uint128,
        delta: Uint128,
        finders_fee_bps: u64,
    },
    CreateNftPool {
        collection: String,
        asset_recipient: Option<String>,
        bonding_curve: BondingCurve,
        spot_price: Uint128,
        delta: Uint128,
        finders_fee_bps: u64,
    },
    CreateTradePool {
        collection: String,
        asset_recipient: Option<String>,
        bonding_curve: BondingCurve,
        spot_price: Uint128,
        delta: Uint128,
        finders_fee_bps: u64,
        swap_fee_bps: u64,
        reinvest_tokens: bool,
        reinvest_nfts: bool,
    },
    /// Deposit tokens into a pool
    DepositTokens {
        pool_id: u64,
    },
    /// Deposit NFTs into a pool
    DepositNfts {
        pool_id: u64,
        collection: String,
        nft_token_ids: Vec<String>,
    },
    /// Withdraw tokens from a pool
    WithdrawTokens {
        pool_id: u64,
        amount: Uint128,
        asset_recipient: Option<String>,
    },
    /// Withdraw all tokens from a pool
    WithdrawAllTokens {
        pool_id: u64,
        asset_recipient: Option<String>,
    },
    /// Withdraw NFTs from a pool
    WithdrawNfts {
        pool_id: u64,
        nft_token_ids: Vec<String>,
        asset_recipient: Option<String>,
    },
    /// Withdraw all NFTs from a pool
    WithdrawAllNfts {
        pool_id: u64,
        asset_recipient: Option<String>,
    },
    /// Update the parameters of a pool
    UpdatePoolConfig {
        pool_id: u64,
        asset_recipient: Option<String>,
        delta: Option<Uint128>,
        spot_price: Option<Uint128>,
        finders_fee_bps: Option<u64>,
        swap_fee_bps: Option<u64>,
        reinvest_tokens: Option<bool>,
        reinvest_nfts: Option<bool>,
    },
    // Activate a pool so that it may begin accepting trades
    SetActivePool {
        is_active: bool,
        pool_id: u64,
    },
    /// Remove a pool from contract storage and indexing
    RemovePool {
        pool_id: u64,
        asset_recipient: Option<String>,
    },
    /// Swap NFTs for tokens directly with a specified pool
    DirectSwapNftsForTokens {
        pool_id: u64,
        nft_orders: Vec<NftOrder>,
        swap_params: SwapParams,
    },
}

/// QueryOptions are used to paginate contract queries
#[cw_serde]
pub struct QueryOptions<T> {
    /// Whether to sort items in ascending or descending order
    pub descending: Option<bool>,
    /// The key to start the query after
    pub start_after: Option<T>,
    // The number of items that will be returned
    pub limit: Option<u32>,
}

#[infinity_module_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the global contract configuration object
    #[returns(ConfigResponse)]
    Config {},
    /// Retrieve pools sorted by their pool id
    #[returns(PoolsResponse)]
    Pools { query_options: QueryOptions<u64> },
    /// Retrieve pools by their pool id
    #[returns(PoolsByIdResponse)]
    PoolsById { pool_ids: Vec<u64> },
    /// Retrieve pools by their owner address
    #[returns(PoolsResponse)]
    PoolsByOwner {
        owner: String,
        query_options: QueryOptions<u64>,
    },
    /// Retrieve the NFT token ids in a pool
    #[returns(NftTokenIdsResponse)]
    PoolNftTokenIds {
        pool_id: u64,
        query_options: QueryOptions<String>,
    },
    /// Retrieve pool quotes sorted by their buy quote price
    #[returns(PoolQuoteResponse)]
    QuotesBuyFromPool {
        collection: String,
        query_options: QueryOptions<(Uint128, u64)>,
    },
    /// Retrieve pool quotes sorted by their sell quote price
    #[returns(PoolQuoteResponse)]
    QuotesSellToPool {
        collection: String,
        query_options: QueryOptions<(Uint128, u64)>,
    },
    /// Simulate a DirectSwapNftsForTokens transaction
    #[returns(SwapResponse)]
    SimDirectSwapNftsForTokens {
        sender: String,
        pool_id: u64,
        nft_orders: Vec<NftOrder>,
        swap_params: SwapParams,
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
pub struct NftTokenIdsResponse {
    pub pool_id: u64,
    pub collection: String,
    pub nft_token_ids: Vec<String>,
}

#[cw_serde]
pub struct PoolQuoteResponse {
    pub pool_quotes: Vec<PoolQuote>,
}
