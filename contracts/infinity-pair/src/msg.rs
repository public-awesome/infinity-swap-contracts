#[cfg_attr(not(debug_assertions), allow(unused_imports))]
use crate::{
    pair::Pair,
    state::{BondingCurve, PairConfig, PairImmutable, PairType, TokenId},
};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Uint128};
use sg_index_query::QueryOptions;

/// Defines whether the end user is buying or selling NFTs
#[cw_serde]
pub enum TransactionType {
    UserSubmitsNfts,
    UserSubmitsTokens,
}

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the infinity global contract
    pub infinity_global: String,
    /// The immutable parameters of the pair
    pub pair_immutable: PairImmutable<String>,
    /// The configuration object for the pair
    pub pair_config: PairConfig<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Deposit NFTs into the pair
    DepositNfts {
        collection: String,
        token_ids: Vec<TokenId>,
    },
    /// Withdraw NFTs from the pair
    WithdrawNfts {
        collection: String,
        token_ids: Vec<TokenId>,
        asset_recipient: Option<String>,
    },
    /// Withdraw any NFTs, from the pair
    WithdrawAnyNfts {
        collection: String,
        limit: u32,
        asset_recipient: Option<String>,
    },
    /// Deposit tokens into the pair
    DepositTokens {},
    /// Withdraw tokens from the pair
    WithdrawTokens {
        funds: Vec<Coin>,
        asset_recipient: Option<String>,
    },
    /// Withdraw all tokens from the pair
    WithdrawAllTokens {
        asset_recipient: Option<String>,
    },
    /// Update the parameters of a pair
    UpdatePairConfig {
        is_active: Option<bool>,
        pair_type: Option<PairType>,
        bonding_curve: Option<BondingCurve>,
        asset_recipient: Option<String>,
    },
    // Swap NFT for Tokens at the pair price
    SwapNftForTokens {
        token_id: String,
        min_output: Coin,
        asset_recipient: Option<String>,
    },
    // Swap Tokens for a specific NFT at the pair price
    SwapTokensForSpecificNft {
        token_id: String,
        asset_recipient: Option<String>,
    },
    // Swap Tokens for any NFT at the pair price
    SwapTokensForAnyNft {
        asset_recipient: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Pair)]
    Pair {},
    #[returns(NftDepositsResponse)]
    NftDeposits {
        query_options: Option<QueryOptions<String>>,
    },
    #[returns(QuotesResponse)]
    SimSellToPairSwaps {
        limit: u32,
    },
    #[returns(QuotesResponse)]
    SimBuyFromPairSwaps {
        limit: u32,
    },
}

#[cw_serde]
pub struct NftDepositsResponse {
    pub collection: Addr,
    pub token_ids: Vec<TokenId>,
}

#[cw_serde]
pub struct QuotesResponse {
    pub denom: String,
    pub sell_to_pair_quotes: Vec<Uint128>,
    pub buy_from_pair_quotes: Vec<Uint128>,
}
