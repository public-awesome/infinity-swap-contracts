use cosmwasm_schema::{cw_serde, QueryResponses};
use infinity_macros::{infinity_module_execute, infinity_module_query};

pub const MAX_QUERY_LIMIT: u32 = 100;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the marketplace contract
    pub marketplace: String,
    /// The max number of NFT swaps that can be processed in a single message
    pub max_batch_size: u32,
}

#[infinity_module_execute]
#[cw_serde]
pub enum ExecuteMsg {}

#[infinity_module_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
