use cosmwasm_schema::{cw_serde, serde::Serialize};
use cosmwasm_std::{to_binary, Addr, Api, Binary, StdError, Timestamp, Uint128};
use cw_utils::maybe_addr;

#[cw_serde]
pub enum TransactionType {
    UserSubmitsNfts,
    UserSubmitsTokens,
}

#[cw_serde]
pub struct TokenPayment {
    pub label: String,
    pub amount: Uint128,
    pub address: String,
}

#[cw_serde]
pub struct NftPayment {
    pub label: String,
    pub token_id: String,
    pub address: String,
}

#[cw_serde]
pub struct Swap {
    pub source: String,
    pub transaction_type: TransactionType,
    pub sale_price: Uint128,
    pub network_fee: Uint128,
    pub nft_payments: Vec<NftPayment>,
    pub token_payments: Vec<TokenPayment>,
    pub data: Option<Binary>,
}

impl Swap {
    pub fn set_data<T: Serialize>(&mut self, data: T) {
        self.data = Some(to_binary(&data).unwrap());
    }
}

#[cw_serde]
pub struct SwapResponse {
    pub swaps: Vec<Swap>,
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

pub struct SwapParamsInternal {
    pub deadline: Timestamp,
    pub robust: bool,
    pub asset_recipient: Option<Addr>,
    pub finder: Option<Addr>,
}

pub fn transform_swap_params(
    api: &dyn Api,
    swap_params: SwapParams,
) -> Result<SwapParamsInternal, StdError> {
    Ok(SwapParamsInternal {
        deadline: swap_params.deadline,
        robust: swap_params.robust,
        asset_recipient: maybe_addr(api, swap_params.asset_recipient)?,
        finder: maybe_addr(api, swap_params.finder)?,
    })
}
