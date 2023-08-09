use crate::{
    pair::Pair,
    state::{BondingCurve, PairConfig, PairImmutable, PairType, TokenId},
};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};
use cw721::Cw721ReceiveMsg;

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
    ReceiveNft(Cw721ReceiveMsg),
    /// Withdraw NFTs from the pair
    WithdrawNfts {
        token_ids: Vec<TokenId>,
    },
    /// Withdraw any NFTs, from the pair
    WithdrawAnyNfts {
        limit: u32,
    },
    /// Deposit tokens into the pair
    DepositTokens {},
    /// Withdraw tokens from the pair
    WithdrawTokens {
        amount: Uint128,
    },
    /// Withdraw all tokens from the pair
    WithdrawAllTokens {},
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
    // /// Remove a pair from contract storage and indexing
    // RemovePair {
    //     pair_id: u64,
    //     asset_recipient: Option<String>,
    // },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Pair)]
    Pair {},
    // #[returns(Vec<TokenId>)]
    // NftDeposits {
    //     query_options: Option<QueryOptions<String>>,
    // },
    // #[returns(Option<Uint128>)]
    // BuyFromPairQuote {},
    // #[returns(Option<Uint128>)]
    // SellToPairQuote {},
}
