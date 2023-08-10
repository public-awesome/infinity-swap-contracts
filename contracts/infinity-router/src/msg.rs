use crate::{
    nfts_for_tokens_iterators::types::{NftForTokensQuote, NftForTokensSource},
    tokens_for_nfts_iterators::types::{TokensForNftQuote, TokensForNftSource},
    ContractError,
};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Api, Uint128};
use cw_address_like::AddressLike;
use cw_utils::maybe_addr;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the infinity global contract
    pub infinity_global: String,
}

/// SwapParams contains the parameters for a swap
#[cw_serde]
#[derive(Default)]
pub struct SwapParams<T: AddressLike> {
    /// Whether or not to revert the entire trade if one of the swaps fails
    pub robust: Option<bool>,
    /// The address to receive the assets from the swap, if not specified is set to sender
    pub asset_recipient: Option<T>,
}

impl SwapParams<String> {
    pub fn str_to_addr(&self, api: &dyn Api) -> Result<SwapParams<Addr>, ContractError> {
        Ok(SwapParams {
            robust: self.robust,
            asset_recipient: maybe_addr(api, self.asset_recipient.clone())?,
        })
    }
}

#[cw_serde]
pub struct SellOrder {
    pub input_token_id: String,
    pub min_output: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    SwapNftsForTokens {
        collection: String,
        denom: String,
        sell_orders: Vec<SellOrder>,
        swap_params: Option<SwapParams<String>>,
        filter_sources: Option<Vec<NftForTokensSource>>,
    },
    SwapTokensForNfts {
        collection: String,
        denom: String,
        max_inputs: Vec<Uint128>,
        swap_params: Option<SwapParams<String>>,
        filter_sources: Option<Vec<TokensForNftSource>>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<NftForTokensQuote>)]
    NftsForTokens {
        collection: String,
        denom: String,
        limit: u32,
        filter_sources: Option<Vec<NftForTokensSource>>,
    },
    #[returns(Vec<TokensForNftQuote>)]
    TokensForNfts {
        collection: String,
        denom: String,
        limit: u32,
        filter_sources: Option<Vec<TokensForNftSource>>,
    },
}
