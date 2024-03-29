#[cfg_attr(not(debug_assertions), allow(unused_imports))]
use crate::state::PairQuote;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use sg_index_query::QueryOptions;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the infinity global contract
    pub infinity_global: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Update the buy and sell quotes in the index for a pair
    UpdatePairIndices {
        collection: String,
        denom: String,
        sell_to_pair_quote: Option<Uint128>,
        buy_from_pair_quote: Option<Uint128>,
    },
}

#[cw_serde]
pub struct PairQuoteOffset {
    /// The address of the infinity pair contract
    pub pair: String,
    /// The amount of tokens in being quoted
    pub amount: Uint128,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<PairQuote>)]
    SellToPairQuotes {
        collection: String,
        denom: String,
        query_options: Option<QueryOptions<PairQuoteOffset>>,
    },
    #[returns(Vec<PairQuote>)]
    BuyFromPairQuotes {
        collection: String,
        denom: String,
        query_options: Option<QueryOptions<PairQuoteOffset>>,
    },
}
