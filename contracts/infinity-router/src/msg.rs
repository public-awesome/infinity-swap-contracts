use cosmwasm_schema::{cw_serde, QueryResponses};

pub const MAX_QUERY_LIMIT: u32 = 100;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the marketplace contract
    pub marketplace: String,
    /// The address of the infinity swap contract
    pub infinity_swap: String,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
