use cosmwasm_schema::{cw_serde, QueryResponses};
use infinity_interface::{NftOrder, SwapParams, SwapResponse};
use infinity_macros::infinity_module_query;

pub const MAX_QUERY_LIMIT: u32 = 100;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the marketplace contract
    pub marketplace: String,
}

#[infinity_module_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SwapResponse)]
    SimSwapTokensForSpecificNfts {
        sender: String,
        collection: String,
        nft_orders: Vec<NftOrder>,
        swap_params: SwapParams,
    },
}
