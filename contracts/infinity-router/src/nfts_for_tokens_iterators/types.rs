use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use infinity_pair::pair::Pair;
use std::cmp::Ordering;

#[cw_serde]
pub enum NftForTokensSource {
    Infinity,
}

#[cw_serde]
pub enum NftForTokensSourceData {
    Infinity(Pair),
}

#[cw_serde]
pub struct NftForTokensQuote {
    pub address: Addr,
    pub amount: Uint128,
    pub source_data: NftForTokensSourceData,
}

impl Ord for NftForTokensQuote {
    fn cmp(&self, other: &Self) -> Ordering {
        self.amount.cmp(&other.amount)
    }
}

impl PartialOrd for NftForTokensQuote {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for NftForTokensQuote {}
