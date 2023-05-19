use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    // The code ID of the infinity global contract
    pub infinity_global_code_id: u64,
    // The code ID of the infinity factory contract
    pub infinity_factory_code_id: u64,
    // The code ID of the infinity index contract
    pub infinity_index_code_id: u64,
    // The code ID of the infinity router contract
    pub infinity_router_code_id: u64,
    // The code ID of the infinity pool contract
    pub infinity_pool_code_id: u64,
    // The address of the marketplace contract
    pub marketplace: String,
    // The minimum price for an NFT
    pub min_price: Uint128,
    // The fee paid when creating a new pool
    pub pool_creation_fee: Uint128,
    // The trading fee paid during NFT transactions
    pub trading_fee_bps: u64,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
