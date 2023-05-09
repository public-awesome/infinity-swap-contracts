use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use infinity_shared::interface::NftOrder;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the infinity index contract
    pub infinity_index: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SwapNftsForTokens {
        collection: String,
        sender: String,
        nft_orders: Vec<NftOrder>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
