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
pub struct TokensForNftQuote {
    pub address: Addr,
    pub amount: Uint128,
    pub source_data: TokensForNftSourceData,
}

impl Ord for TokensForNftQuote {
    fn cmp(&self, other: &Self) -> Ordering {
        self.amount.cmp(&other.amount)
    }
}

impl PartialOrd for TokensForNftQuote {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TokensForNftQuote {}
