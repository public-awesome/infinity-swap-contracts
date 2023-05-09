use crate::state::PoolQuote;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the global gov contract
    pub global_gov: String,
    /// The address of the infinity factory contract
    pub infinity_factory: String,
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
    QuoteSellToPool {
        collection: String,
        limit: u64,
    },
}

#[cw_serde]
pub struct PoolQuoteResponse {
    pub pool_quotes: Vec<PoolQuote>,
}
