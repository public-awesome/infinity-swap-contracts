use crate::{
    state::{BondingCurve, PairType, SudoParams},
    ContractError,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, Decimal, Timestamp, Uint128};
use cw_address_like::AddressLike;
use cw_utils::maybe_addr;
use std::fmt;

#[cw_serde]
pub struct InstantiateMsg {
    /// The parameters used to configure the protocol
    pub sudo_params: SudoParams<String>,
}

/// SwapParams contains the parameters for a swap
#[cw_serde]
pub struct SwapParams {
    /// The address to receive the assets from the swap, if not specified is set to sender
    pub asset_recipient: Option<String>,
    /// The address of the finder, will receive a portion of the fees equal to percentage set by the pair
    pub finder: Option<String>,
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

/// PairNftSwap is the parent of NftSwap and organizes swaps by pair_id
#[cw_serde]
pub struct PairNftSwap {
    /// The id of the pair to swap in
    pub pair_id: u64,
    /// The NFT swaps to execute
    pub nft_swaps: Vec<NftSwap>,
}

#[cw_serde]
pub struct PairOptions<T: AddressLike> {
    pub asset_recipient: Option<T>,
    pub finders_fee_percent: Option<Decimal>,
}

impl<T: AddressLike> Default for PairOptions<T> {
    fn default() -> Self {
        PairOptions {
            asset_recipient: None,
            finders_fee_percent: None,
        }
    }
}

impl PairOptions<String> {
    pub fn str_to_addr(self, api: &dyn Api) -> Result<PairOptions<Addr>, ContractError> {
        Ok(PairOptions {
            asset_recipient: maybe_addr(api, self.asset_recipient)?,
            finders_fee_percent: self.finders_fee_percent,
        })
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Create a new pair, defaults to an inactive state
    CreatePair {
        collection: String,
        denom: String,
        pair_type: PairType,
        bonding_curve: BondingCurve,
        pair_options: Option<PairOptions<String>>,
    },
    /// Deposit tokens into a pair
    DepositTokens { pair_id: u64 },
    /// Deposit NFTs into a pair
    DepositNfts {
        pair_id: u64,
        collection: String,
        nft_token_ids: Vec<String>,
    },
    /// Withdraw tokens from a pair
    WithdrawTokens { pair_id: u64, amount: Uint128 },
    /// Withdraw all tokens from a pair
    WithdrawAllTokens { pair_id: u64 },
    /// Withdraw NFTs from a pair
    WithdrawNfts {
        pair_id: u64,
        nft_token_ids: Vec<String>,
    },
    /// Withdraw all NFTs from a pair
    WithdrawAllNfts { pair_id: u64 },
    // /// Update the parameters of a pair
    // UpdatePairConfig {
    //     pair_id: u64,
    //     asset_recipient: Option<String>,
    //     delta: Option<Uint128>,
    //     spot_price: Option<Uint128>,
    //     finders_fee_bps: Option<u64>,
    //     swap_fee_bps: Option<u64>,
    //     reinvest_tokens: Option<bool>,
    //     reinvest_nfts: Option<bool>,
    // },
    // // Activate a pair so that it may begin accepting trades
    // SetActivePair {
    //     is_active: bool,
    //     pair_id: u64,
    // },
    // /// Remove a pair from contract storage and indexing
    // RemovePair {
    //     pair_id: u64,
    //     asset_recipient: Option<String>,
    // },
    // /// Swap NFTs for tokens directly with a specified pair
    // DirectSwapNftsForTokens {
    //     pair_id: u64,
    //     nfts_to_swap: Vec<NftSwap>,
    //     swap_params: SwapParams,
    // },
    // /// Swap NFTs for tokens at optimal sale prices
    // SwapNftsForTokens {
    //     collection: String,
    //     nfts_to_swap: Vec<NftSwap>,
    //     swap_params: SwapParams,
    // },
    // /// Swap tokens for NFTs directly with a specified pair
    // /// Note: client must specify which NFTs they want to swap for
    // DirectSwapTokensForSpecificNfts {
    //     pair_id: u64,
    //     nfts_to_swap_for: Vec<NftSwap>,
    //     swap_params: SwapParams,
    // },
    // /// Swap tokens for specific NFTs at optimal purchase prices
    // SwapTokensForSpecificNfts {
    //     collection: String,
    //     pair_nfts_to_swap_for: Vec<PairNftSwap>,
    //     swap_params: SwapParams,
    // },
    // /// Swap tokens for any NFTs at optimal purchase prices
    // SwapTokensForAnyNfts {
    //     collection: String,
    //     max_expected_token_input: Vec<Uint128>,
    //     swap_params: SwapParams,
    // },
}

#[cw_serde]
pub enum QueryMsg {}

// #[cw_serde]
// pub enum QueryMsg {
//     /// Get the global contract configuration object
//     /// Return type: `ConfigResponse`
//     Config {},
//     /// Retrieve pairs sorted by their pair id
//     /// Return type: `PairsResponse`
//     Pairs { query_options: QueryOptions<u64> },
//     /// Retrieve pairs by their pair id
//     /// Return type: `PairsByIdResponse`
//     PairsById { pair_ids: Vec<u64> },
//     /// Retrieve pairs by their owner address
//     /// Return type: `PairsResponse`
//     PairsByOwner {
//         owner: String,
//         query_options: QueryOptions<u64>,
//     },
//     /// Retrieve the NFT token ids in a pair
//     /// Return type: `NftTokenIdsResponse`
//     PairNftTokenIds {
//         pair_id: u64,
//         query_options: QueryOptions<String>,
//     },
//     /// Retrieve pair quotes sorted by their buy quote price
//     /// Return type: `PairQuoteResponse`
//     QuotesBuyFromPair {
//         collection: String,
//         query_options: QueryOptions<(Uint128, u64)>,
//     },
//     /// Retrieve pair quotes sorted by their sell quote price
//     /// Return type: `PairQuoteResponse`
//     QuotesSellToPair {
//         collection: String,
//         query_options: QueryOptions<(Uint128, u64)>,
//     },
//     /// Simulate a DirectSwapNftsForTokens transaction
//     /// Return type: `SwapResponse`
//     SimDirectSwapNftsForTokens {
//         pair_id: u64,
//         nfts_to_swap: Vec<NftSwap>,
//         sender: String,
//         swap_params: SwapParams,
//     },
//     /// Simulate a SwapNftsForTokens transaction
//     /// Return type: `SwapResponse`
//     SimSwapNftsForTokens {
//         collection: String,
//         nfts_to_swap: Vec<NftSwap>,
//         sender: String,
//         swap_params: SwapParams,
//     },
//     /// Simulate a DirectSwapTokensforSpecificNfts transaction
//     /// Return type: `SwapResponse`
//     SimDirectSwapTokensForSpecificNfts {
//         pair_id: u64,
//         nfts_to_swap_for: Vec<NftSwap>,
//         sender: String,
//         swap_params: SwapParams,
//     },
//     /// Simulate a SimSwapTokensForSpecificNfts transaction
//     /// Return type: `SwapResponse`
//     SimSwapTokensForSpecificNfts {
//         collection: String,
//         pair_nfts_to_swap_for: Vec<PairNftSwap>,
//         sender: String,
//         swap_params: SwapParams,
//     },
//     /// Simulate a SwapTokensForAnyNfts transaction
//     /// Return type: `SwapResponse`
//     SimSwapTokensForAnyNfts {
//         collection: String,
//         max_expected_token_input: Vec<Uint128>,
//         sender: String,
//         swap_params: SwapParams,
//     },
// }

// #[cw_serde]
// pub struct ConfigResponse {
//     pub config: Config,
// }

// #[cw_serde]
// pub struct PairsResponse {
//     pub pairs: Vec<Pair>,
// }

// #[cw_serde]
// pub struct PairsByIdResponse {
//     pub pairs: Vec<(u64, Option<Pair>)>,
// }

// #[cw_serde]
// pub struct NftTokenIdsResponse {
//     pub pair_id: u64,
//     pub nft_token_ids: Vec<String>,
// }

// #[cw_serde]
// pub struct PairQuoteResponse {
//     pub pair_quotes: Vec<PairQuote>,
// }

// #[cw_serde]
// pub struct SwapResponse {
//     pub swaps: Vec<Swap>,
// }
