use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use infinity_pair::pair::Pair;
use std::cmp::Ordering;

#[cw_serde]
pub enum TokensForNftSource {
    Infinity,
}

#[cw_serde]
pub enum TokensForNftSourceData {
    Infinity(Pair),
}

#[cw_serde]
pub struct TokensForNftInternal {
    pub address: Addr,
    pub amount: Uint128,
    pub source_data: TokensForNftSourceData,
}

impl Ord for TokensForNftInternal {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.amount, &self.address).cmp(&(&other.amount, &other.address))
    }
}

impl PartialOrd for TokensForNftInternal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TokensForNftInternal {}

#[cw_serde]
pub struct TokensForNftQuote {
    pub address: Addr,
    pub amount: Uint128,
    pub source: TokensForNftSource,
}

impl From<&TokensForNftInternal> for TokensForNftQuote {
    fn from(internal: &TokensForNftInternal) -> Self {
        TokensForNftQuote {
            address: internal.address.clone(),
            amount: internal.amount,
            source: match &internal.source_data {
                TokensForNftSourceData::Infinity(_) => TokensForNftSource::Infinity,
            },
        }
    }
}
