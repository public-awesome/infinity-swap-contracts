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
pub struct NftForTokensInternal {
    pub address: Addr,
    pub amount: Uint128,
    pub source_data: NftForTokensSourceData,
}

impl Ord for NftForTokensInternal {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.amount, &self.address).cmp(&(&other.amount, &other.address))
    }
}

impl PartialOrd for NftForTokensInternal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for NftForTokensInternal {}

#[cw_serde]
pub struct NftForTokensQuote {
    pub address: Addr,
    pub amount: Uint128,
    pub source: NftForTokensSource,
}

impl From<&NftForTokensInternal> for NftForTokensQuote {
    fn from(internal: &NftForTokensInternal) -> Self {
        NftForTokensQuote {
            address: internal.address.clone(),
            amount: internal.amount,
            source: match &internal.source_data {
                NftForTokensSourceData::Infinity(_) => NftForTokensSource::Infinity,
            },
        }
    }
}
