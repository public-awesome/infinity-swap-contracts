use crate::state::GlobalConfig;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    // The address of the infinity index contract
    pub infinity_index: String,
    // The address of the infinity factory contract
    pub infinity_factory: String,
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
pub enum QueryMsg {
    #[returns(GlobalConfigResponse)]
    GlobalConfig {},
}

#[cw_serde]
pub struct GlobalConfigResponse {
    pub config: GlobalConfig,
}

#[cw_serde]
pub enum SudoMsg {
    UpdateConfig {
        infinity_index: Option<String>,
        infinity_factory: Option<String>,
        min_price: Option<Uint128>,
        pool_creation_fee: Option<Uint128>,
        trading_fee_bps: Option<u64>,
    },
}
