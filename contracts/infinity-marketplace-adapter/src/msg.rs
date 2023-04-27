use cosmwasm_schema::{cw_serde, QueryResponses};
use infinity_shared::interface::{NftOrder, SwapParams, SwapResponse};

pub const MAX_QUERY_LIMIT: u32 = 100;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the marketplace contract
    pub marketplace: String,
    /// The max number of NFT swaps that can be processed in a single message
    pub max_batch_size: u32,
}

#[cw_serde]
pub enum ExecuteMsg {
    SwapNftsForTokens {
        collection: String,
        nft_orders: Vec<NftOrder>,
        swap_params: SwapParams,
    },
    SwapTokensForSpecificNfts {
        collection: String,
        nft_orders: Vec<NftOrder>,
        swap_params: SwapParams,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SwapResponse)]
    SimSwapNftsForTokens {
        sender: String,
        collection: String,
        nft_orders: Vec<NftOrder>,
        swap_params: SwapParams,
    },
    #[returns(SwapResponse)]
    SimSwapTokensForSpecificNfts {
        sender: String,
        collection: String,
        nft_orders: Vec<NftOrder>,
        swap_params: SwapParams,
    },
}
