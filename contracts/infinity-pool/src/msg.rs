use crate::{
    state::{BondingCurve, Config, Pool, PoolQuote},
    swap_processor::Swap,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint128};
use std::fmt;

#[cw_serde]
pub struct InstantiateMsg {
    /// The fungible token used in the pools
    pub denom: String,
    /// The address of the marketplace contract
    pub marketplace_addr: String,
    /// The address of the developer who will receive a portion of the fair burn
    pub developer: Option<String>,
}

/// SwapParams contains the parameters for a swap
#[cw_serde]
pub struct SwapParams {
    /// The deadline after which the swap will be rejected
    pub deadline: Timestamp,
    /// Whether or not to revert the entire trade if one of the swaps fails
    pub robust: bool,
    /// The address to receive the assets from the swap, if not specified is set to sender
    pub asset_recipient: Option<String>,
    /// The address of the finder, will receive a portion of the fees equal to percentage set by the pool
    pub finder: Option<String>,
}

/// Defines whether the end user is buying or selling NFTs
#[cw_serde]
pub enum TransactionType {
    NftsForTokens,
    TokensForNfts,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// NftSwap contains the parameters for an NFT swap
#[cw_serde]
pub struct NftSwap {
    /// The id of the NFT to swap
    pub nft_token_id: String,
    /// The amount of tokens to accept in exchange for the NFT
    /// Note: this could be the minimum acceptable amount for a sale
    /// or the maximum acceptable amount for a purchase
    pub token_amount: Uint128,
}

/// PoolNftSwap is the parent of NftSwap and organizes swaps by pool_id
#[cw_serde]
pub struct PoolNftSwap {
    /// The id of the pool to swap in
    pub pool_id: u64,
    /// The NFT swaps to execute
    pub nft_swaps: Vec<NftSwap>,
}

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
        nfts_to_swap: Vec<NftSwap>,
        swap_params: SwapParams,
    },
    /// Swap NFTs for tokens at optimal sale prices
    SwapNftsForTokens {
        collection: String,
        nfts_to_swap: Vec<NftSwap>,
        swap_params: SwapParams,
    },
    /// Swap tokens for NFTs directly with a specified pool
    /// Note: client must specify which NFTs they want to swap for
    DirectSwapTokensForSpecificNfts {
        pool_id: u64,
        nfts_to_swap_for: Vec<NftSwap>,
        swap_params: SwapParams,
    },
    /// Swap tokens for specific NFTs at optimal purchase prices
    SwapTokensForSpecificNfts {
        collection: String,
        pool_nfts_to_swap_for: Vec<PoolNftSwap>,
        swap_params: SwapParams,
    },
    /// Swap tokens for any NFTs at optimal purchase prices
    SwapTokensForAnyNfts {
        collection: String,
        max_expected_token_input: Vec<Uint128>,
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

#[cw_serde]
pub enum QueryMsg {
    /// Get the global contract configuration object
    /// Return type: `ConfigResponse`
    Config {},
    /// Retrieve pools sorted by their pool id
    /// Return type: `PoolsResponse`
    Pools { query_options: QueryOptions<u64> },
    /// Retrieve pools by their pool id
    /// Return type: `PoolsByIdResponse`
    PoolsById { pool_ids: Vec<u64> },
    /// Retrieve pools by their owner address
    /// Return type: `PoolsResponse`
    PoolsByOwner {
        owner: String,
        query_options: QueryOptions<u64>,
    },
    /// Retrieve pool quotes sorted by their buy quote price
    /// Return type: `PoolQuoteResponse`
    PoolQuotesBuy {
        collection: String,
        query_options: QueryOptions<(Uint128, u64)>,
    },
    /// Retrieve pool quotes sorted by their sell quote price
    /// Return type: `PoolQuoteResponse`
    PoolQuotesSell {
        collection: String,
        query_options: QueryOptions<(Uint128, u64)>,
    },
    /// Simulate a DirectSwapNftsForTokens transaction
    /// Return type: `SwapResponse`
    SimDirectSwapNftsForTokens {
        pool_id: u64,
        nfts_to_swap: Vec<NftSwap>,
        sender: String,
        swap_params: SwapParams,
    },
    /// Simulate a SwapNftsForTokens transaction
    /// Return type: `SwapResponse`
    SimSwapNftsForTokens {
        collection: String,
        nfts_to_swap: Vec<NftSwap>,
        sender: String,
        swap_params: SwapParams,
    },
    /// Simulate a DirectSwapTokensforSpecificNfts transaction
    /// Return type: `SwapResponse`
    SimDirectSwapTokensforSpecificNfts {
        pool_id: u64,
        nfts_to_swap_for: Vec<NftSwap>,
        sender: String,
        swap_params: SwapParams,
    },
    /// Simulate a SimSwapTokensForSpecificNfts transaction
    /// Return type: `SwapResponse`
    SimSwapTokensForSpecificNfts {
        collection: String,
        pool_nfts_to_swap_for: Vec<PoolNftSwap>,
        sender: String,
        swap_params: SwapParams,
    },
    /// Simulate a SwapTokensForAnyNfts transaction
    /// Return type: `SwapResponse`
    SimSwapTokensForAnyNfts {
        collection: String,
        max_expected_token_input: Vec<Uint128>,
        sender: String,
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
pub struct PoolQuoteResponse {
    pub pool_quotes: Vec<PoolQuote>,
}

#[cw_serde]
pub struct SwapResponse {
    pub swaps: Vec<Swap>,
}
