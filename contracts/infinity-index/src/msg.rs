use crate::state::PoolQuote;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use infinity_shared::query::QueryOptions;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the infinity global contract
    pub infinity_global: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateBuyFromPoolQuote {
        collection: String,
        quote_price: Option<Uint128>,
    },
    UpdateSellToPoolQuote {
        collection: String,
        quote_price: Option<Uint128>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(PoolQuoteResponse)]
    BuyFromPoolQuotes {
        collection: String,
        query_options: Option<QueryOptions<(u128, String)>>,
    },
    #[returns(PoolQuoteResponse)]
    SellToPoolQuotes {
        collection: String,
        query_options: Option<QueryOptions<(u128, String)>>,
    },
}

#[cw_serde]
pub struct PoolQuoteResponse {
    pub pool_quotes: Vec<PoolQuote>,
}
