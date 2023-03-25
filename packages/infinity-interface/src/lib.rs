use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint128};

#[cw_serde]
pub enum TransactionType {
    NftsForTokens,
    TokensForNfts,
}

#[cw_serde]
pub struct TokenPayment {
    pub label: String,
    pub amount: Uint128,
    pub address: String,
}

#[cw_serde]
pub struct NftPayment {
    pub token_id: String,
    pub address: String,
}

#[cw_serde]
pub struct Swap {
    pub transaction_type: TransactionType,
    pub sale_price: Uint128,
    pub network_fee: Uint128,
    pub nft_payment: NftPayment,
    pub token_payments: Vec<TokenPayment>,
}

#[cw_serde]
pub struct SwapResponse {
    pub swap: Vec<Swap>,
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
