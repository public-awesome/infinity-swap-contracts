use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Timestamp, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the infinity global contract
    pub infinity_global: String,
}

#[cw_serde]
pub struct NftOrder {
    pub token_id: String,
    pub amount: Uint128,
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
    /// The address of the finder, will receive a portion of the fees equal to percentage set by maker
    pub finder: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    SwapNftsForTokens {
        collection: String,
        nft_orders: Vec<NftOrder>,
        swap_params: SwapParams,
    },
    SwapNftsForTokensInternal {},
    SwapTokensForSpecificNfts {
        collection: String,
        nft_orders: Vec<NftOrder>,
        swap_params: SwapParams,
    },
    SwapTokensForSpecificNftsInternal {},
    SwapTokensForAnyNfts {
        collection: String,
        orders: Vec<Uint128>,
        swap_params: SwapParams,
    },
    SwapTokensForAnyNftsInternal {},
    CleanupSwapContext {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
